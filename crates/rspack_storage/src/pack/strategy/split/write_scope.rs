use async_trait::async_trait;
use futures::future::join_all;
use futures::TryFutureExt;
use itertools::Itertools;
use rayon::iter::{IntoParallelIterator, ParallelBridge, ParallelIterator};
use rspack_error::{error, Result};
use rspack_paths::Utf8PathBuf;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::{util::choose_bucket, SplitPackStrategy};
use crate::pack::{
  data::{Pack, PackScope},
  strategy::{PackWriteStrategy, ScopeUpdate, ScopeWriteStrategy, WriteScopeResult},
};

#[async_trait]
impl ScopeWriteStrategy for SplitPackStrategy {
  fn before_all(&self, _: &mut PackScope) -> Result<()> {
    Ok(())
  }

  async fn before_write(&self, scope: &PackScope) -> Result<()> {
    let temp_path = self.get_temp_path(&scope.path)?;
    self.fs.remove_dir(&temp_path).await?;
    self.fs.ensure_dir(&temp_path).await?;
    self.fs.ensure_dir(&scope.path).await?;
    Ok(())
  }

  async fn after_write(
    &self,
    scope: &PackScope,
    wrote_files: HashSet<Utf8PathBuf>,
    removed_files: HashSet<Utf8PathBuf>,
  ) -> Result<()> {
    self.remove_files(removed_files).await?;
    self.move_temp_files(wrote_files).await?;
    self.remove_unrelated_files(scope).await?;
    self.fs.remove_dir(&self.temp_root).await?;
    Ok(())
  }

  fn after_all(&self, scope: &mut PackScope) -> Result<()> {
    let scope_meta = scope.meta.expect_value_mut();
    for bucket in scope_meta.packs.iter_mut() {
      for pack in bucket {
        pack.wrote = true;
      }
    }
    Ok(())
  }

  fn update_scope(&self, scope: &mut PackScope, updates: ScopeUpdate) -> Result<()> {
    if !scope.loaded() {
      return Err(error!("scope not loaded, run `load` first"));
    }
    let mut scope_meta = scope.meta.take_value().expect("should have scope meta");
    let mut scope_packs = scope.packs.take_value().expect("should have scope packs");

    // get changed buckets
    let bucket_updates = updates
      .into_par_iter()
      .map(|(key, value)| {
        let bucket_id = choose_bucket(&key, &scope.options.bucket_size);
        (bucket_id, key, value)
      })
      .collect::<Vec<_>>()
      .into_iter()
      .fold(
        HashMap::<usize, ScopeUpdate>::default(),
        |mut res, (bucket_id, key, value)| {
          res.entry(bucket_id).or_default().insert(key, value);
          res
        },
      );

    let changed_buckets = bucket_updates
      .into_iter()
      .map(|(bucket_id, bucket_update)| {
        let old_metas = std::mem::take(
          scope_meta
            .packs
            .get_mut(bucket_id)
            .expect("should have bucket pack metas"),
        );

        let old_packs = std::mem::take(
          scope_packs
            .get_mut(bucket_id)
            .expect("should have bucket packs"),
        );

        let bucket_packs = old_metas
          .into_iter()
          .zip(old_packs)
          .collect::<HashMap<_, _>>();

        (bucket_id, bucket_update, bucket_packs)
      })
      .par_bridge()
      .map(|(bucket_id, bucket_update, bucket_packs)| {
        let packs = self.update_packs(
          scope.path.join(bucket_id.to_string()),
          scope.options.as_ref(),
          bucket_packs,
          bucket_update,
        );
        (bucket_id, packs)
      })
      .collect::<Vec<_>>();

    let mut remain_files = HashSet::default();
    for (bucket_id, bucket_result) in changed_buckets {
      for (pack_meta, pack) in bucket_result.remain_packs {
        remain_files.insert(pack.path.clone());
        scope_packs[bucket_id].push(pack);
        scope_meta.packs[bucket_id].push(pack_meta);
      }

      for (pack_meta, pack) in bucket_result.new_packs {
        scope_packs[bucket_id].push(pack);
        scope_meta.packs[bucket_id].push(pack_meta);
      }

      scope.removed.extend(bucket_result.removed_files);
    }

    scope.removed.retain(|r| !remain_files.contains(r));

    scope.packs.set_value(scope_packs);
    scope.meta.set_value(scope_meta);

    Ok(())
  }

  async fn write_packs(&self, scope: &mut PackScope) -> Result<WriteScopeResult> {
    if !scope.loaded() {
      return Err(error!("scope not loaded, run `load` first"));
    }
    let removed_files = std::mem::take(&mut scope.removed);
    let packs = scope.packs.take_value().expect("should have scope packs");
    let meta = scope.meta.expect_value_mut();

    let mut wrote_files = HashSet::default();

    let (wrote_pack_infos, new_pack_infos): (Vec<_>, Vec<_>) = packs
      .into_iter()
      .flatten()
      .zip(meta.packs.iter_mut().flatten())
      .partition(|x| x.1.wrote);

    let (new_packs, new_pack_metas): (Vec<_>, Vec<_>) = new_pack_infos.into_iter().unzip();
    let write_results = batch_write_packs(new_packs, self).await?;

    let mut wrote_packs = wrote_pack_infos
      .into_iter()
      .map(|(pack, meta)| (meta.hash.clone(), pack))
      .collect::<HashMap<_, _>>();

    for (meta, (hash, pack)) in new_pack_metas.into_iter().zip(write_results.into_iter()) {
      meta.hash = hash.clone();
      meta.size = pack.size();
      wrote_files.insert(pack.path.clone());
      wrote_packs.insert(hash, pack);
    }

    let mut wrote_scope_packs = vec![];
    for bucket_pack_metas in meta.packs.iter() {
      let mut bucket_packs = vec![];
      for pack_meta in bucket_pack_metas {
        let pack = wrote_packs
          .remove(&pack_meta.hash)
          .expect("should have pack");
        bucket_packs.push(pack);
      }

      wrote_scope_packs.push(bucket_packs);
    }

    scope.packs.set_value(wrote_scope_packs);

    Ok(WriteScopeResult {
      wrote_files,
      removed_files,
    })
  }

  async fn write_meta(&self, scope: &mut PackScope) -> Result<WriteScopeResult> {
    if !scope.loaded() {
      return Err(error!("scope not loaded, run `load` first"));
    }
    let meta = scope.meta.expect_value();
    let path = self.get_temp_path(&meta.path)?;
    self
      .fs
      .ensure_dir(path.parent().expect("should have parent"))
      .await?;

    let mut writer = self.fs.write_file(&path).await?;

    writer
      .write_line(
        format!(
          "{} {} {}",
          meta.bucket_size, meta.pack_size, meta.last_modified
        )
        .as_str(),
      )
      .await?;

    for bucket_id in 0..meta.bucket_size {
      let line = meta
        .packs
        .get(bucket_id)
        .map(|packs| {
          packs
            .iter()
            .map(|meta| format!("{},{},{}", meta.name, meta.hash, meta.size))
            .join(" ")
        })
        .unwrap_or_default();
      writer.write_line(&line).await?;
    }

    writer.flush().await?;

    Ok(WriteScopeResult {
      wrote_files: HashSet::from_iter(vec![meta.path.clone()]),
      removed_files: Default::default(),
    })
  }
}

async fn save_pack(pack: &Pack, strategy: &SplitPackStrategy) -> Result<String> {
  let keys = pack.keys.expect_value();
  let contents = pack.contents.expect_value();
  if keys.len() != contents.len() {
    return Err(error!("pack keys and contents length not match"));
  }
  strategy.write_pack(pack).await?;
  let hash = strategy
    .get_pack_hash(&strategy.get_temp_path(&pack.path)?, keys, contents)
    .await?;
  Ok(hash)
}

async fn batch_write_packs(
  packs: Vec<Pack>,
  strategy: &SplitPackStrategy,
) -> Result<Vec<(String, Pack)>> {
  let tasks = packs.into_iter().map(|pack| {
    let strategy = strategy.to_owned();
    tokio::spawn(async move { (save_pack(&pack, &strategy).await, pack) })
      .map_err(|e| error!("{}", e))
  });

  let task_result = join_all(tasks)
    .await
    .into_iter()
    .collect::<Result<Vec<(Result<String>, Pack)>>>()?;

  let mut res = vec![];
  for (hash, pack) in task_result {
    res.push((hash?, pack));
  }
  Ok(res)
}

#[cfg(test)]
mod tests {
  use std::{collections::HashMap, sync::Arc};

  use rspack_error::Result;

  use crate::pack::{
    data::{PackOptions, PackScope},
    strategy::{
      split::util::test_pack_utils::{
        clean_strategy, count_bucket_packs, count_scope_packs, create_strategies,
        get_bucket_pack_sizes, mock_updates, save_scope, UpdateVal,
      },
      ScopeReadStrategy, ScopeWriteStrategy,
    },
    SplitPackStrategy,
  };

  async fn test_short_value(
    scope: &mut PackScope,
    strategy: &SplitPackStrategy,
    start: usize,
    end: usize,
  ) -> Result<()> {
    let updates = mock_updates(start, end, 8, UpdateVal::Value("val".into()));
    strategy.update_scope(scope, updates)?;
    let contents = scope
      .get_contents()
      .into_iter()
      .map(|(k, v)| (k.as_ref().to_owned(), v.as_ref().to_owned()))
      .collect::<HashMap<_, _>>();

    assert_eq!(contents.len(), end);
    assert_eq!(
      *contents
        .get(format!("{:0>4}_key", start).as_bytes())
        .expect("should have key"),
      format!("{:0>4}_val", start).as_bytes()
    );
    assert_eq!(
      *contents
        .get(format!("{:0>4}_key", end - 1).as_bytes())
        .expect("should have key"),
      format!("{:0>4}_val", end - 1).as_bytes()
    );

    Ok(())
  }

  async fn test_long_value(
    scope: &mut PackScope,
    strategy: &SplitPackStrategy,
    start: usize,
    end: usize,
  ) -> Result<()> {
    let updates = mock_updates(start, end, 24, UpdateVal::Value("val".into()));
    let pre_item_count = scope.get_contents().len();
    strategy.update_scope(scope, updates)?;
    let contents = scope
      .get_contents()
      .into_iter()
      .map(|(k, v)| (k.as_ref().to_owned(), v.as_ref().to_owned()))
      .collect::<HashMap<_, _>>();

    assert_eq!(contents.len(), pre_item_count + end - start);
    assert_eq!(
      *contents
        .get(format!("{:0>20}_key", start).as_bytes())
        .expect("should have key"),
      format!("{:0>20}_val", start).as_bytes()
    );
    assert_eq!(
      *contents
        .get(format!("{:0>20}_key", end - 1).as_bytes())
        .expect("should have key"),
      format!("{:0>20}_val", end - 1).as_bytes()
    );
    Ok(())
  }

  async fn test_update_value(scope: &mut PackScope, strategy: &SplitPackStrategy) -> Result<()> {
    let updates = mock_updates(0, 1, 8, UpdateVal::Value("new".into()));
    let pre_item_count = scope.get_contents().len();
    strategy.update_scope(scope, updates)?;
    let contents = scope
      .get_contents()
      .into_iter()
      .map(|(k, v)| (k.as_ref().to_owned(), v.as_ref().to_owned()))
      .collect::<HashMap<_, _>>();

    assert_eq!(contents.len(), pre_item_count);
    assert_eq!(
      *contents
        .get(format!("{:0>4}_key", 0).as_bytes())
        .expect("should have key"),
      format!("{:0>4}_new", 0).as_bytes()
    );

    Ok(())
  }

  async fn test_remove_value(scope: &mut PackScope, strategy: &SplitPackStrategy) -> Result<()> {
    let updates = mock_updates(1, 2, 8, UpdateVal::Removed);
    let pre_item_count = scope.get_contents().len();
    strategy.update_scope(scope, updates)?;
    let contents = scope
      .get_contents()
      .into_iter()
      .map(|(k, v)| (k.as_ref().to_owned(), v.as_ref().to_owned()))
      .collect::<HashMap<_, _>>();

    assert_eq!(contents.len(), pre_item_count - 1);
    assert!(!contents.contains_key(format!("{:0>4}_key", 1).as_bytes()));
    Ok(())
  }

  async fn test_single_bucket(scope: &mut PackScope, strategy: &SplitPackStrategy) -> Result<()> {
    test_short_value(scope, strategy, 0, 10).await?;
    assert_eq!(count_scope_packs(scope), 5);
    let res = save_scope(scope, strategy).await?;
    // 5 packs + 1 meta
    assert_eq!(res.wrote_files.len(), 6);
    assert_eq!(res.removed_files.len(), 0);

    test_long_value(scope, strategy, 10, 15).await?;
    assert_eq!(count_scope_packs(scope), 10);
    let res = save_scope(scope, strategy).await?;
    // 5 packs + 1 meta
    assert_eq!(res.wrote_files.len(), 6);
    assert_eq!(res.removed_files.len(), 0);

    test_update_value(scope, strategy).await?;
    assert_eq!(count_scope_packs(scope), 10);
    let res = save_scope(scope, strategy).await?;
    // 1 pack + 1 meta
    assert_eq!(res.wrote_files.len(), 2);
    assert_eq!(res.removed_files.len(), 1);

    test_remove_value(scope, strategy).await?;
    assert_eq!(count_scope_packs(scope), 10);
    let res = save_scope(scope, strategy).await?;
    // 1 pack + 1 meta
    assert_eq!(res.wrote_files.len(), 2);
    assert_eq!(res.removed_files.len(), 1);

    Ok(())
  }

  async fn test_multi_bucket(scope: &mut PackScope, strategy: &SplitPackStrategy) -> Result<()> {
    test_short_value(scope, strategy, 0, 100).await?;
    assert_eq!(count_bucket_packs(scope), vec![5; 10]);

    let res = save_scope(scope, strategy).await?;
    // 50 packs + 1 meta
    assert_eq!(res.wrote_files.len(), 51);
    assert_eq!(res.removed_files.len(), 0);

    test_long_value(scope, strategy, 100, 150).await?;
    assert_eq!(count_bucket_packs(scope), vec![10; 10]);
    let res = save_scope(scope, strategy).await?;
    // 50 packs + 1 meta
    assert_eq!(res.wrote_files.len(), 51);
    assert_eq!(res.removed_files.len(), 0);

    test_update_value(scope, strategy).await?;
    assert_eq!(count_bucket_packs(scope), vec![10; 10]);
    let res = save_scope(scope, strategy).await?;
    // 1 packs + 1 meta
    assert_eq!(res.wrote_files.len(), 2);
    assert_eq!(res.removed_files.len(), 1);

    test_remove_value(scope, strategy).await?;
    assert_eq!(count_bucket_packs(scope), vec![10; 10]);
    let res = save_scope(scope, strategy).await?;
    // 1 packs + 1 meta
    assert_eq!(res.wrote_files.len(), 2);
    assert_eq!(res.removed_files.len(), 1);

    Ok(())
  }

  async fn test_big_bucket(scope: &mut PackScope, strategy: &SplitPackStrategy) -> Result<()> {
    // 200 * 16 = 3200 = 2000 + 1200
    test_short_value(scope, strategy, 0, 200).await?;
    assert_eq!(count_scope_packs(scope), 2);
    assert_eq!(get_bucket_pack_sizes(scope), [1200, 2000]);

    // 3200 + 100 * 16 = 4800 = 2000 + 2000 + 800
    test_short_value(scope, strategy, 200, 300).await?;
    assert_eq!(count_scope_packs(scope), 3);
    assert_eq!(get_bucket_pack_sizes(scope), [800, 2000, 2000]);

    // 4800 + 60 * 16 = 5760 = 2000 + 2000 + 1760(>1600)
    test_short_value(scope, strategy, 300, 360).await?;
    assert_eq!(count_scope_packs(scope), 3);
    assert_eq!(get_bucket_pack_sizes(scope), [1760, 2000, 2000]);

    // 5760 + 160 = 5920 = 2000 + 2000 + 1760(>1600) + 160
    test_short_value(scope, strategy, 360, 370).await?;
    assert_eq!(count_scope_packs(scope), 4);
    assert_eq!(get_bucket_pack_sizes(scope), [160, 1760, 2000, 2000]);

    Ok(())
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn should_write_single_bucket_scope() {
    for strategy in create_strategies("write_single") {
      let options = Arc::new(PackOptions {
        bucket_size: 1,
        pack_size: 32,
        expire: 1000000,
      });
      let mut scope = PackScope::empty(strategy.get_path("scope_name"), options.clone());
      clean_strategy(&strategy).await;

      let _ = test_single_bucket(&mut scope, &strategy)
        .await
        .map_err(|e| {
          panic!("{}", e);
        });
    }
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn should_write_multi_bucket_scope() {
    for strategy in create_strategies("write_multi") {
      let options = Arc::new(PackOptions {
        bucket_size: 10,
        pack_size: 32,
        expire: 1000000,
      });
      let mut scope = PackScope::empty(strategy.get_path("scope_name"), options.clone());
      clean_strategy(&strategy).await;

      let _ = test_multi_bucket(&mut scope, &strategy).await.map_err(|e| {
        panic!("{}", e);
      });
    }
  }

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn should_write_big_bucket_scope() {
    for strategy in create_strategies("write_big") {
      let options = Arc::new(PackOptions {
        bucket_size: 1,
        pack_size: 2000,
        expire: 1000000,
      });
      let mut scope = PackScope::empty(strategy.get_path("scope_name"), options.clone());
      clean_strategy(&strategy).await;

      let _ = test_big_bucket(&mut scope, &strategy).await.map_err(|e| {
        panic!("{}", e);
      });
    }
  }
}
