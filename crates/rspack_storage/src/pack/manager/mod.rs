mod queue;

use std::sync::Arc;

use futures::future::join_all;
use pollster::block_on;
use queue::TaskQueue;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use tokio::sync::oneshot::Receiver;
use tokio::sync::{oneshot, Mutex};

use super::data::{PackOptions, PackScope, RootMeta, RootMetaState, RootOptions};
use super::strategy::{ScopeStrategy, WriteScopeResult};
use super::ScopeUpdates;
use crate::error::{Error, ErrorType, ValidateResult};
use crate::{ItemPairs, Result};

type ScopeMap = HashMap<String, PackScope>;

#[derive(Debug)]
pub struct ScopeManager {
  pub root_options: Arc<RootOptions>,
  pub pack_options: Arc<PackOptions>,
  pub strategy: Arc<dyn ScopeStrategy>,
  pub scopes: Arc<Mutex<ScopeMap>>,
  pub root_meta: Arc<Mutex<RootMetaState>>,
  pub queue: TaskQueue,
}

impl ScopeManager {
  pub fn new(
    root_options: Arc<RootOptions>,
    pack_options: Arc<PackOptions>,
    strategy: Arc<dyn ScopeStrategy>,
  ) -> Self {
    ScopeManager {
      root_options,
      pack_options,
      strategy,
      scopes: Default::default(),
      queue: TaskQueue::new(),
      root_meta: Default::default(),
    }
  }

  pub fn save(&self, updates: ScopeUpdates) -> Result<Receiver<Result<()>>> {
    let pack_options = self.pack_options.clone();
    let strategy = self.strategy.clone();
    let scopes = self.scopes.clone();
    let root_meta = self.root_meta.clone();
    block_on(tokio::task::unconstrained(async move {
      let mut scopes_guard = scopes.lock().await;
      match update_scopes(
        &mut scopes_guard,
        updates,
        pack_options.clone(),
        strategy.as_ref(),
      )
      .await
      {
        Ok(_) => {
          *root_meta.lock().await = RootMetaState::Value(Some(RootMeta::new(
            scopes_guard
              .iter()
              .filter(|(_, scope)| scope.loaded())
              .map(|(name, _)| name.clone())
              .collect::<HashSet<_>>(),
            self.root_options.expire,
          )));
          Ok(())
        }
        Err(e) => Err(e),
      }
    }))?;

    let strategy = self.strategy.clone();
    let scopes = self.scopes.clone();
    let root_meta = self.root_meta.clone();
    let root_options = self.root_options.clone();
    let (tx, rx) = oneshot::channel();
    self.queue.add_task(Box::pin(async move {
      let mut scopes_lock = scopes.lock().await;
      let root_meta = root_meta
        .lock()
        .await
        .expect_value()
        .clone()
        .expect("should have root meta");
      let old_scopes = std::mem::take(&mut *scopes_lock);
      let res = save_scopes(old_scopes, &root_meta, strategy.as_ref(), &root_options).await;
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

  async fn clear_scope(&self, name: &str) {
    self
      .scopes
      .lock()
      .await
      .get_mut(name)
      .expect("should have scope")
      .clear();
  }

  pub async fn load(&self, name: &'static str) -> Result<ItemPairs> {
    self
      .scopes
      .lock()
      .await
      .entry(name.to_string())
      .or_insert_with(|| {
        PackScope::new(
          name,
          self.strategy.get_path(name),
          self.pack_options.clone(),
        )
      });

    // only check lock file and root meta for the first time
    if matches!(*self.root_meta.lock().await, RootMetaState::Pending) {
      match self.strategy.before_load().await {
        Ok(()) => {
          let loaded = self.strategy.read_root_meta().await?;
          *self.root_meta.lock().await = RootMetaState::Value(loaded);
        }
        Err(err) => {
          *self.root_meta.lock().await = RootMetaState::Value(None);
          self.clear_scope(name).await;
          return Err(err);
        }
      }
    }

    match self.validate_scope(name).await {
      Ok(validated) => match validated {
        // load from disk for valid scope
        ValidateResult::Valid => {
          let mut scopes = self.scopes.lock().await;
          let scope = scopes.get_mut(name).expect("should have scope");
          self.strategy.ensure_contents(scope).await?;
          let res = scope.get_contents();
          self.strategy.release_scope(scope).await?;
          Ok(res)
        }
        // create empty scope if not exists
        ValidateResult::NotExists => {
          self.clear_scope(name).await;
          Ok(vec![])
        }
        // clear scope if invalid
        ValidateResult::Invalid(detail) => {
          self.clear_scope(name).await;
          Err(Error::from_detail(
            Some(ErrorType::Validate),
            Some(name),
            detail,
          ))
        }
      },
      Err(e) => {
        // clear scope if error
        self.clear_scope(name).await;
        Err(Error::from_error(
          Some(ErrorType::Validate),
          Some(name),
          Box::new(e),
        ))
      }
    }
  }

  async fn validate_scope(&self, name: &'static str) -> Result<ValidateResult> {
    let root_meta_guard = self.root_meta.lock().await;
    // no root, no scope
    let Some(root_meta) = root_meta_guard.expect_value() else {
      return Ok(ValidateResult::NotExists);
    };
    // no scope, need to create a new one
    let mut scopes_guard = self.scopes.lock().await;
    let Some(scope) = scopes_guard.get_mut(name) else {
      return Ok(ValidateResult::NotExists);
    };

    // scope exists, validate it
    let validated = self.strategy.validate_root(root_meta).await?;
    if validated.is_valid() {
      self.strategy.ensure_meta(scope).await?;
      let validated = self.strategy.validate_meta(scope).await?;
      if validated.is_valid() {
        self.strategy.ensure_keys(scope).await?;
        self.strategy.validate_packs(scope).await
      } else {
        Ok(validated)
      }
    } else {
      Ok(validated)
    }
  }
}

#[tracing::instrument("Cache::Storage::update_scopes", skip_all)]
async fn update_scopes(
  scopes: &mut ScopeMap,
  mut updates: ScopeUpdates,
  pack_options: Arc<PackOptions>,
  strategy: &dyn ScopeStrategy,
) -> Result<()> {
  for (scope_name, _) in updates.iter() {
    scopes.entry(scope_name.to_string()).or_insert_with(|| {
      PackScope::empty(
        scope_name,
        strategy.get_path(scope_name),
        pack_options.clone(),
      )
    });
  }

  join_all(
    scopes
      .iter_mut()
      .filter_map(|(name, scope)| {
        updates
          .remove(name.to_string().as_str())
          .and_then(|scope_update| {
            if scope_update.is_empty() {
              None
            } else {
              Some((scope, scope_update))
            }
          })
      })
      .map(|(scope, scope_update)| strategy.update_scope(scope, scope_update)),
  )
  .await
  .into_iter()
  .collect::<Result<Vec<_>>>()?;

  Ok(())
}

#[tracing::instrument("Cache::Storage::save_scopes", skip_all)]
async fn save_scopes(
  mut scopes: ScopeMap,
  root_meta: &RootMeta,
  strategy: &dyn ScopeStrategy,
  root_options: &RootOptions,
) -> Result<ScopeMap> {
  scopes.retain(|_, scope| scope.loaded());

  strategy.before_all(&mut scopes).await?;

  join_all(
    scopes
      .values_mut()
      .map(|scope| strategy.optimize_scope(scope)),
  )
  .await
  .into_iter()
  .collect::<Result<Vec<_>>>()?;

  let changed = join_all(scopes.values_mut().map(|scope| async move {
    let mut res = WriteScopeResult::default();
    if scope.loaded() {
      res.extend(strategy.write_packs(scope).await?);
      res.extend(strategy.write_meta(scope).await?);
      strategy.release_scope(scope).await?;
    }
    Ok(res)
  }))
  .await
  .into_iter()
  .collect::<Result<Vec<WriteScopeResult>>>()?
  .into_iter()
  .fold(WriteScopeResult::default(), |mut acc, res| {
    acc.extend(res);
    acc
  });

  strategy.write_root_meta(root_meta).await?;
  strategy.merge_changed(changed).await?;
  strategy.after_all(&mut scopes).await?;
  strategy.clean(root_meta, &scopes, root_options).await?;

  Ok(scopes.into_iter().collect())
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use rspack_fs::MemoryFileSystem;
  use rspack_paths::{Utf8Path, Utf8PathBuf};
  use rustc_hash::FxHashMap as HashMap;

  use crate::{
    error::Result,
    pack::{
      data::{PackOptions, RootOptions},
      manager::ScopeManager,
      strategy::SplitPackStrategy,
    },
    BridgeFileSystem, FileSystem, ItemKey, ItemValue,
  };

  fn mock_key(id: usize) -> ItemKey {
    format!("{:0>length$}_key", id, length = 46)
      .as_bytes()
      .to_vec()
  }

  fn mock_insert_value(id: usize) -> Option<ItemValue> {
    Some(
      format!("{:0>length$}_val", id, length = 46)
        .as_bytes()
        .to_vec(),
    )
  }

  fn mock_update_value(id: usize) -> Option<ItemValue> {
    Some(
      format!("{:0>length$}_new", id, length = 46)
        .as_bytes()
        .to_vec(),
    )
  }

  async fn test_cold_start(
    root: &Utf8Path,
    temp: &Utf8Path,
    fs: Arc<dyn FileSystem>,
  ) -> Result<()> {
    let root_options = Arc::new(RootOptions {
      expire: 60000,
      root: root.parent().expect("should get parent").to_path_buf(),
      clean: true,
    });
    let pack_options = Arc::new(PackOptions {
      bucket_size: 10,
      pack_size: 500,
    });

    let strategy = Arc::new(SplitPackStrategy::new(
      root.to_path_buf(),
      temp.to_path_buf(),
      fs.clone(),
      Some(1),
      Some(2),
    ));
    let manager = ScopeManager::new(root_options, pack_options, strategy);

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

    assert!(fs.exists(root.join("storage_meta").as_path()).await?);
    Ok(())
  }

  async fn test_hot_start(root: &Utf8Path, temp: &Utf8Path, fs: Arc<dyn FileSystem>) -> Result<()> {
    let root_options = Arc::new(RootOptions {
      expire: 60000,
      root: root.parent().expect("should get parent").to_path_buf(),
      clean: true,
    });
    let pack_options = Arc::new(PackOptions {
      bucket_size: 10,
      pack_size: 500,
    });

    let strategy = Arc::new(SplitPackStrategy::new(
      root.to_path_buf(),
      temp.to_path_buf(),
      fs.clone(),
      Some(1),
      Some(2),
    ));
    let manager = ScopeManager::new(root_options, pack_options, strategy);

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

  async fn test_invalid_start(
    root: &Utf8Path,
    temp: &Utf8Path,
    fs: Arc<dyn FileSystem>,
  ) -> Result<()> {
    let root_options = Arc::new(RootOptions {
      expire: 60000,
      root: root.parent().expect("should get parent").to_path_buf(),
      clean: true,
    });
    let pack_options = Arc::new(PackOptions {
      // different bucket size
      bucket_size: 100,
      pack_size: 500,
    });

    let strategy = Arc::new(SplitPackStrategy::new(
      root.to_path_buf(),
      temp.to_path_buf(),
      fs.clone(),
      Some(1),
      Some(2),
    ));
    let manager = ScopeManager::new(root_options.clone(), pack_options.clone(), strategy.clone());
    // should report error when invalid failed
    assert_eq!(
      manager.load("scope1").await.unwrap_err().to_string(),
      "validate scope `scope1` failed due to `options.bucketSize` changed"
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
    // assert_eq!(manager.load("scope1").await?.len(), 100);
    rx.await
      .unwrap_or_else(|e| panic!("save failed: {:?}", e))?;
    // assert_eq!(manager.load("scope1").await?.len(), 100);

    // // will override cache files to new one
    // let manager2 = ScopeManager::new(root_options, pack_options, strategy);
    // assert_eq!(manager2.load("scope1").await?.len(), 100);

    Ok(())
  }

  async fn test_manager() -> Result<()> {
    let fs = Arc::new(BridgeFileSystem(Arc::new(MemoryFileSystem::default())));
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
