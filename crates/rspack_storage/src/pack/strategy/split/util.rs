use std::hash::Hasher;

use itertools::Itertools;
use rustc_hash::FxHasher;

use crate::pack::data::{Pack, PackContents, PackFileMeta, PackKeys, PackScope};

pub type PackIndexList = Vec<(usize, usize)>;
pub type PackInfoList<'a> = Vec<(&'a PackFileMeta, &'a Pack)>;

pub fn flag_scope_wrote(scope: &mut PackScope) {
  let scope_meta = scope.meta.expect_value_mut();
  for bucket in scope_meta.packs.iter_mut() {
    for pack in bucket {
      pack.wrote = true;
    }
  }
}

pub fn get_indexed_packs<'a>(
  scope: &'a PackScope,
  filter: Option<&dyn Fn(&'a Pack, &'a PackFileMeta) -> bool>,
) -> (PackIndexList, PackInfoList<'a>) {
  let meta = scope.meta.expect_value();
  let packs = scope.packs.expect_value();

  meta
    .packs
    .iter()
    .enumerate()
    .flat_map(|(bucket_id, pack_meta_list)| {
      let bucket_packs = packs.get(bucket_id).expect("should have bucket packs");
      pack_meta_list
        .iter()
        .enumerate()
        .map(|(pack_pos, pack_meta)| {
          (
            (bucket_id, pack_pos),
            (
              pack_meta,
              bucket_packs.get(pack_pos).expect("should have bucket pack"),
            ),
          )
        })
        .collect_vec()
    })
    .filter(|(_, (pack_meta, pack))| {
      if let Some(filter) = filter {
        filter(pack, pack_meta)
      } else {
        true
      }
    })
    .unzip()
}

pub fn get_name(keys: &PackKeys, _: &PackContents) -> String {
  let mut hasher = FxHasher::default();
  for k in keys {
    hasher.write(k);
  }
  hasher.write_usize(keys.len());

  format!("{:016x}", hasher.finish())
}

pub fn choose_bucket(key: &[u8], total: &usize) -> usize {
  let num = key.iter().fold(0_usize, |acc, i| acc + *i as usize);
  num % total
}

#[cfg(test)]
pub mod test_pack_utils {
  use std::sync::Arc;

  use itertools::Itertools;
  use rspack_fs::{MemoryFileSystem, NativeFileSystem};
  use rspack_paths::{AssertUtf8, Utf8Path, Utf8PathBuf};
  use rustc_hash::FxHashMap as HashMap;

  use super::flag_scope_wrote;
  use crate::{
    BridgeFileSystem, FileSystem,
    error::Result,
    pack::{
      data::{PackOptions, PackScope, current_time},
      strategy::{
        ScopeUpdate, SplitPackStrategy, WriteScopeResult, split::handle_file::prepare_scope,
      },
    },
  };

  pub async fn mock_root_meta_file(path: &Utf8Path, fs: &dyn FileSystem) -> Result<()> {
    fs.ensure_dir(path.parent().expect("should have parent"))
      .await?;
    let mut writer = fs.write_file(path).await?;
    let current = current_time();
    writer.write_all(current.to_string().as_bytes()).await?;
    writer.flush().await?;

    Ok(())
  }

  pub async fn mock_scope_meta_file(
    path: &Utf8Path,
    fs: &dyn FileSystem,
    options: &PackOptions,
    pack_count: usize,
  ) -> Result<()> {
    let generation = 1_usize;
    fs.ensure_dir(path.parent().expect("should have parent"))
      .await?;
    let mut writer = fs.write_file(path).await?;
    writer
      .write_line(
        format!(
          "{} {} {}",
          options.bucket_size, options.pack_size, generation
        )
        .as_str(),
      )
      .await?;
    for bucket_id in 0..options.bucket_size {
      let mut pack_meta_list = vec![];
      for pack_no in 0..pack_count {
        let pack_name = format!("pack_name_{bucket_id}_{pack_no}");
        let pack_hash = format!("pack_hash_{bucket_id}_{pack_no}");
        let pack_size = 100;
        pack_meta_list.push(format!("{pack_name},{pack_hash},{pack_size},{generation}"));
      }
      writer.write_line(pack_meta_list.join(" ").as_str()).await?;
    }

    writer.flush().await?;

    Ok(())
  }

  pub async fn mock_pack_file(
    path: &Utf8Path,
    unique_id: &str,
    item_count: usize,
    fs: &dyn FileSystem,
  ) -> Result<()> {
    fs.ensure_dir(path.parent().expect("should have parent"))
      .await?;
    let mut writer = fs.write_file(path).await?;
    let mut keys = vec![];
    let mut contents = vec![];
    let generations = vec![1_usize; item_count];
    for i in 0..item_count {
      keys.push(format!("key_{unique_id}_{i}").as_bytes().to_vec());
      contents.push(format!("val_{unique_id}_{i}").as_bytes().to_vec());
    }
    writer
      .write_line(keys.iter().map(|k| k.len()).join(" ").as_str())
      .await?;
    writer
      .write_line(contents.iter().map(|k| k.len()).join(" ").as_str())
      .await?;
    writer
      .write_line(generations.into_iter().join(" ").as_str())
      .await?;
    for key in keys {
      writer.write(&key).await?;
    }
    for content in contents {
      writer.write(&content).await?;
    }
    writer.flush().await?;
    Ok(())
  }

  pub enum UpdateVal {
    Value(String),
    Removed,
  }

  pub fn mock_updates(start: usize, end: usize, length: usize, value: UpdateVal) -> ScopeUpdate {
    let mut updates = HashMap::default();
    for i in start..end {
      updates.insert(
        format!("{:0>length$}_key", i, length = length - 4)
          .as_bytes()
          .to_vec(),
        match &value {
          UpdateVal::Value(str) => Some(
            format!("{:0>length$}_{}", i, str, length = length - (str.len() + 1))
              .as_bytes()
              .to_vec(),
          ),
          UpdateVal::Removed => None,
        },
      );
    }

    updates
  }

  pub fn count_scope_packs(scope: &PackScope) -> usize {
    scope.packs.expect_value().iter().flatten().count()
  }
  pub fn count_bucket_packs(scope: &PackScope) -> Vec<usize> {
    scope
      .packs
      .expect_value()
      .iter()
      .map(|i| i.len())
      .collect_vec()
  }
  pub fn get_bucket_pack_sizes(scope: &PackScope) -> Vec<usize> {
    let mut res = scope
      .meta
      .expect_value()
      .packs
      .iter()
      .flatten()
      .map(|c| c.size)
      .collect_vec();
    res.sort_unstable();
    res
  }

  pub async fn clean_strategy(strategy: &SplitPackStrategy) {
    strategy
      .fs
      .remove_dir(&strategy.root)
      .await
      .expect("should remove dir");
    strategy
      .fs
      .remove_dir(&strategy.temp_root)
      .await
      .expect("should remove dir");
  }

  pub async fn flush_file_mtime(path: &Utf8Path, fs: Arc<dyn FileSystem>) -> Result<()> {
    let content = fs.read_file(path).await?.read_to_end().await?;
    let mut writer = fs.write_file(path).await?;
    writer.write_all(&content).await?;
    writer.flush().await?;

    Ok(())
  }

  pub async fn save_scope(
    scope: &mut PackScope,
    strategy: &SplitPackStrategy,
  ) -> Result<WriteScopeResult> {
    prepare_scope(
      &scope.path,
      &strategy.root,
      &strategy.temp_root,
      strategy.fs.clone(),
    )
    .await?;

    let mut changed = WriteScopeResult::default();
    changed.extend(strategy.write_packs(scope).await?);
    changed.extend(strategy.write_meta(scope).await?);
    strategy.release_scope(scope).await?;
    strategy.merge_changed(changed.clone()).await?;
    flag_scope_wrote(scope);

    Ok(changed)
  }

  pub fn get_native_path(p: &str) -> Utf8PathBuf {
    std::env::temp_dir()
      .join("./rspack_test/storage/pack_strategy/")
      .join(p)
      .assert_utf8()
  }

  pub fn get_memory_path(p: &str) -> Utf8PathBuf {
    Utf8PathBuf::from("/pack_strategy/").join(p)
  }

  pub fn create_strategies(case: &str) -> Vec<SplitPackStrategy> {
    let fs = [
      (
        Arc::new(BridgeFileSystem(Arc::new(MemoryFileSystem::default()))),
        get_memory_path(case),
      ),
      (
        Arc::new(BridgeFileSystem(Arc::new(NativeFileSystem::new(false)))),
        get_native_path(case),
      ),
    ];

    fs.into_iter()
      .map(|(fs, root)| {
        SplitPackStrategy::new(
          root.join("cache"),
          root.join("temp"),
          fs,
          Some(1_usize),
          Some(2_usize),
        )
      })
      .collect_vec()
  }
}
