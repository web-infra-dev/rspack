mod queue;

use std::sync::Arc;

use futures::future::join_all;
use itertools::Itertools;
use pollster::block_on;
use queue::TaskQueue;
use rayon::iter::{ParallelBridge, ParallelIterator};
use rspack_error::{error, Error, Result};
use rustc_hash::FxHashMap as HashMap;
use tokio::sync::oneshot::Receiver;
use tokio::sync::{oneshot, Mutex};

use super::data::{PackOptions, PackScope};
use super::strategy::{ScopeStrategy, ValidateResult, WriteScopeResult};
use super::ScopeUpdates;
use crate::StorageContent;

type ScopeMap = HashMap<&'static str, PackScope>;

#[derive(Debug)]
pub struct ScopeManager {
  pub strategy: Arc<dyn ScopeStrategy>,
  pub options: Arc<PackOptions>,
  pub scopes: Arc<Mutex<ScopeMap>>,
  pub queue: TaskQueue,
  pub expire: u64,
}

impl ScopeManager {
  pub fn new(options: Arc<PackOptions>, strategy: Arc<dyn ScopeStrategy>, expire: u64) -> Self {
    ScopeManager {
      strategy,
      options,
      scopes: Default::default(),
      queue: TaskQueue::new(),
      expire,
    }
  }

  pub fn save(&self, updates: ScopeUpdates) -> Result<Receiver<Result<()>>> {
    let options = self.options.clone();
    let strategy = self.strategy.clone();
    let scopes = self.scopes.clone();
    block_on(async move {
      update_scopes(
        &mut *scopes.lock().await,
        updates,
        options.clone(),
        strategy.as_ref(),
      )
    })?;

    let strategy = self.strategy.clone();
    let scopes = self.scopes.clone();
    let (tx, rx) = oneshot::channel();
    self.queue.add_task(Box::pin(async move {
      let mut scopes_lock = scopes.lock().await;
      let old_scopes = std::mem::take(&mut *scopes_lock);
      let res = save_scopes(old_scopes, strategy.as_ref()).await;
      let _ = match res {
        Ok(new_scopes) => {
          let _ = std::mem::replace(&mut *scopes_lock, new_scopes);
          tx.send(Ok(()))
        }
        Err(e) => tx.send(Err(e)),
      };
    }));

    Ok(rx)
  }

  pub async fn load(&self, name: &'static str) -> Result<StorageContent> {
    let mut scopes = self.scopes.lock().await;
    let scope = scopes
      .entry(name)
      .or_insert_with(|| PackScope::new(self.strategy.get_path(name), self.options.clone()));

    match validate_scope(scope, self.strategy.as_ref(), self.expire).await {
      Ok(validated) => match validated {
        ValidateResult::Valid => {
          self.strategy.ensure_contents(scope).await?;
          Ok(scope.get_contents())
        }
        ValidateResult::NotExists => {
          scope.clear();
          Ok(vec![])
        }
        ValidateResult::Invalid(_) => {
          scope.clear();
          Err(error!(validated.to_string()))
        }
      },
      Err(e) => {
        scope.clear();
        Err(Error::from(e))
      }
    }
  }
}

async fn validate_scope(
  scope: &mut PackScope,
  strategy: &dyn ScopeStrategy,
  expire: u64,
) -> Result<ValidateResult> {
  if let Some(root_meta) = strategy.read_root_meta().await? {
    let validated = strategy.validate_root(&root_meta, expire).await?;
    if validated.is_valid() {
      strategy.ensure_meta(scope).await?;
      let validated = strategy.validate_meta(scope).await?;
      if validated.is_valid() {
        strategy.ensure_keys(scope).await?;
        strategy.validate_packs(scope).await
      } else {
        Ok(validated)
      }
    } else {
      Ok(validated)
    }
  } else {
    Ok(ValidateResult::NotExists)
  }
}

fn update_scopes(
  scopes: &mut ScopeMap,
  mut updates: ScopeUpdates,
  options: Arc<PackOptions>,
  strategy: &dyn ScopeStrategy,
) -> Result<()> {
  for (scope_name, _) in updates.iter() {
    scopes
      .entry(scope_name)
      .or_insert_with(|| PackScope::empty(strategy.get_path(scope_name), options.clone()));
  }

  scopes
    .iter_mut()
    .filter_map(|(name, scope)| {
      updates
        .remove(name)
        .map(|scope_update| (scope, scope_update))
    })
    .par_bridge()
    .map(|(scope, scope_update)| strategy.update_scope(scope, scope_update))
    .collect::<Result<Vec<_>>>()?;

  Ok(())
}

async fn save_scopes(mut scopes: ScopeMap, strategy: &dyn ScopeStrategy) -> Result<ScopeMap> {
  for (_, scope) in scopes.iter_mut() {
    strategy.before_all(scope)?;
  }

  join_all(
    scopes
      .values()
      .map(|scope| async move { strategy.before_write(scope).await })
      .collect_vec(),
  )
  .await
  .into_iter()
  .collect::<Result<Vec<_>>>()?;

  let wrote_results = join_all(
    scopes
      .values_mut()
      .map(|scope| async move {
        let mut res = WriteScopeResult::default();
        res.extend(strategy.write_packs(scope).await?);
        res.extend(strategy.write_meta(scope).await?);
        Ok(res)
      })
      .collect_vec(),
  )
  .await
  .into_iter()
  .collect::<Result<Vec<WriteScopeResult>>>()?
  .into_iter()
  .collect_vec();

  join_all(
    scopes
      .values()
      .zip(wrote_results)
      .map(|(scope, scope_wrote_result)| async move {
        strategy
          .after_write(
            scope,
            scope_wrote_result.wrote_files,
            scope_wrote_result.removed_files,
          )
          .await
      })
      .collect_vec(),
  )
  .await
  .into_iter()
  .collect::<Result<Vec<_>>>()?;

  strategy.write_root_meta().await?;

  for (_, scope) in scopes.iter_mut() {
    strategy.after_all(scope)?;
  }

  Ok(scopes.into_iter().collect())
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use rspack_error::Result;
  use rspack_fs::MemoryFileSystem;
  use rspack_paths::{Utf8Path, Utf8PathBuf};
  use rustc_hash::FxHashMap as HashMap;

  use crate::{
    pack::{
      data::PackOptions,
      fs::{PackBridgeFS, PackFS},
      manager::ScopeManager,
      strategy::SplitPackStrategy,
    },
    StorageItemKey, StorageItemValue,
  };

  fn mock_key(id: usize) -> StorageItemKey {
    format!("{:0>length$}_key", id, length = 46)
      .as_bytes()
      .to_vec()
  }

  fn mock_insert_value(id: usize) -> Option<StorageItemValue> {
    Some(
      format!("{:0>length$}_val", id, length = 46)
        .as_bytes()
        .to_vec(),
    )
  }

  fn mock_update_value(id: usize) -> Option<StorageItemValue> {
    Some(
      format!("{:0>length$}_new", id, length = 46)
        .as_bytes()
        .to_vec(),
    )
  }

  async fn test_cold_start(root: &Utf8Path, temp: &Utf8Path, fs: Arc<dyn PackFS>) -> Result<()> {
    let options = Arc::new(PackOptions {
      bucket_size: 10,
      pack_size: 500,
    });

    let strategy = Arc::new(SplitPackStrategy::new(
      root.to_path_buf(),
      temp.to_path_buf(),
      fs.clone(),
    ));
    let manager = ScopeManager::new(options, strategy, 60000_u64);

    // start with empty
    assert!(manager.load("scope1").await?.is_empty());
    // update memory scopes but not write to files
    let mut scope_updates = HashMap::default();
    scope_updates.insert(
      "scope1",
      (0..100)
        .map(|i| (mock_key(i), mock_insert_value(i)))
        .collect::<HashMap<_, _>>(),
    );
    scope_updates.insert(
      "scope2",
      (0..100)
        .map(|i| (mock_key(i), mock_insert_value(i)))
        .collect::<HashMap<_, _>>(),
    );
    let rx = manager.save(scope_updates)?;

    assert_eq!(manager.load("scope1").await?.len(), 100);
    assert_eq!(manager.load("scope2").await?.len(), 100);
    assert!(!(fs.exists(root.join("scope1/scope_meta").as_path()).await?));
    assert!(!(fs.exists(root.join("scope2/scope_meta").as_path()).await?));

    // wait for saving to files
    rx.await
      .unwrap_or_else(|e| panic!("save failed: {:?}", e))?;

    assert_eq!(manager.load("scope1").await?.len(), 100);
    assert_eq!(manager.load("scope2").await?.len(), 100);
    assert!(fs.exists(root.join("scope1/scope_meta").as_path()).await?);
    assert!(fs.exists(root.join("scope2/scope_meta").as_path()).await?);
    Ok(())
  }

  async fn test_hot_start(root: &Utf8Path, temp: &Utf8Path, fs: Arc<dyn PackFS>) -> Result<()> {
    let options = Arc::new(PackOptions {
      bucket_size: 10,
      pack_size: 500,
    });

    let strategy = Arc::new(SplitPackStrategy::new(
      root.to_path_buf(),
      temp.to_path_buf(),
      fs.clone(),
    ));
    let manager = ScopeManager::new(options, strategy, 60000_u64);

    // read from files
    assert_eq!(manager.load("scope1").await?.len(), 100);
    assert_eq!(manager.load("scope2").await?.len(), 100);

    // update scopes
    let mut scope_updates = HashMap::default();
    scope_updates.insert(
      "scope1",
      (0..50)
        .map(|i| (mock_key(i), mock_update_value(i)))
        .collect::<HashMap<_, _>>(),
    );
    scope_updates.insert(
      "scope2",
      (0..50)
        .map(|i| (mock_key(i), None))
        .collect::<HashMap<_, _>>(),
    );
    let rx = manager.save(scope_updates)?;
    let (update_items, insert_items): (Vec<_>, Vec<_>) = manager
      .load("scope1")
      .await?
      .into_iter()
      .partition(|(_, v)| {
        let val = String::from_utf8(v.to_vec()).unwrap();
        val.ends_with("_new")
      });
    assert_eq!(insert_items.len(), 50);
    assert_eq!(update_items.len(), 50);
    assert_eq!(manager.load("scope2").await?.len(), 50);
    let scope1_mtime = fs
      .metadata(root.join("scope1/scope_meta").as_path())
      .await?
      .mtime_ms;
    let scope2_meta = fs
      .metadata(root.join("scope2/scope_meta").as_path())
      .await?
      .mtime_ms;

    // wait for updating files
    rx.await
      .unwrap_or_else(|e| panic!("save failed: {:?}", e))?;
    assert_eq!(manager.load("scope1").await?.len(), 100);
    assert_eq!(manager.load("scope2").await?.len(), 50);
    assert_ne!(
      fs.metadata(root.join("scope1/scope_meta").as_path())
        .await?
        .mtime_ms,
      scope1_mtime
    );
    assert_ne!(
      fs.metadata(root.join("scope2/scope_meta").as_path())
        .await?
        .mtime_ms,
      scope2_meta
    );

    Ok(())
  }

  async fn test_invalid_start(root: &Utf8Path, temp: &Utf8Path, fs: Arc<dyn PackFS>) -> Result<()> {
    let options = Arc::new(PackOptions {
      // different bucket size
      bucket_size: 100,
      pack_size: 500,
    });

    let strategy = Arc::new(SplitPackStrategy::new(
      root.to_path_buf(),
      temp.to_path_buf(),
      fs.clone(),
    ));
    let manager = ScopeManager::new(options.clone(), strategy.clone(), 60000_u64);
    // should report error when invalid failed
    assert_eq!(
      manager.load("scope1").await.unwrap_err().to_string(),
      "validation failed due to `options.bucketSize` changed"
    );

    // clear after invalid, can be used as a empty scope
    assert!(manager.load("scope1").await?.is_empty());
    let mut scope_updates = HashMap::default();
    scope_updates.insert(
      "scope1",
      (0..100)
        .map(|i| (mock_key(i), mock_update_value(i)))
        .collect::<HashMap<_, _>>(),
    );
    let rx = manager.save(scope_updates)?;
    assert_eq!(manager.load("scope1").await?.len(), 100);
    rx.await
      .unwrap_or_else(|e| panic!("save failed: {:?}", e))?;
    assert_eq!(manager.load("scope1").await?.len(), 100);

    // will override cache files to new one
    let manager2 = ScopeManager::new(options, strategy, 60000_u64);
    assert_eq!(manager2.load("scope1").await?.len(), 100);

    Ok(())
  }

  async fn test_manager() -> Result<()> {
    let fs = Arc::new(PackBridgeFS(Arc::new(MemoryFileSystem::default())));
    let root = Utf8PathBuf::from("/cache/test_manager");
    let temp = Utf8PathBuf::from("/temp/test_manager");
    test_cold_start(&root, &temp, fs.clone()).await?;
    test_hot_start(&root, &temp, fs.clone()).await?;
    test_invalid_start(&root, &temp, fs.clone()).await?;
    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn should_manager_work() {
    let _ = test_manager().await.map_err(|e| panic!("{:?}", e));
  }
}
