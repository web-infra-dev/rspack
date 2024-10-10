mod option;
mod strategy;

use std::{
  path::PathBuf,
  time::{SystemTime, UNIX_EPOCH},
};

use rspack_cacheable::{from_bytes, to_bytes};
use rustc_hash::FxHashSet as HashSet;

pub use self::option::{PathMatcher, SnapshotOption};
use self::strategy::{Strategy, StrategyHelper, ValidateResult};
use super::storage::ArcStorage;

const SCOPE: &'static str = "snapshot";

#[derive(Debug)]
pub struct Snapshot {
  storage: ArcStorage,
  option: SnapshotOption,
}

impl Snapshot {
  pub fn new(storage: ArcStorage, option: SnapshotOption) -> Self {
    Self { storage, option }
  }

  pub fn add(&self, files: impl Iterator<Item = &PathBuf>) {
    let compiler_time = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_secs();
    let mut helper = StrategyHelper::default();
    for path in files {
      if !path.exists() {
        // TODO Check why non-existent files are being sent here
        continue;
      }
      let path_str = path.to_str().expect("should can convert to string");
      if self.option.is_immutable_path(path_str) {
        continue;
      }
      if self.option.is_managed_path(path_str) {
        if let Some(s) = helper.lib_version(&path) {
          self.storage.set(
            SCOPE,
            path_str.as_bytes().to_vec(),
            to_bytes::<_, ()>(&Strategy::LibVersion(s), &mut ()).unwrap(),
          );
        }
      }
      // compiler time
      self.storage.set(
        SCOPE,
        path_str.as_bytes().to_vec(),
        to_bytes::<_, ()>(&Strategy::CompileTime(compiler_time), &mut ()).unwrap(),
      );
    }
  }

  pub fn remove(&self, files: impl Iterator<Item = &PathBuf>) {
    for item in files {
      self
        .storage
        .remove(SCOPE, item.to_str().expect("should have str").as_bytes())
    }
  }

  pub fn calc_modified_files(&self) -> (HashSet<PathBuf>, HashSet<PathBuf>) {
    let mut helper = StrategyHelper::default();
    let mut modified_files = HashSet::default();
    let mut deleted_files = HashSet::default();

    for (key, value) in self.storage.get_all(SCOPE) {
      let path = PathBuf::from(String::from_utf8(key).unwrap());
      let strategy: Strategy = from_bytes::<Strategy, ()>(&value, &mut ()).unwrap();
      match helper.validate(&path, &strategy) {
        ValidateResult::Modified => {
          modified_files.insert(path);
        }
        ValidateResult::Deleted => {
          deleted_files.insert(path);
        }
        _ => {}
      }
    }
    (modified_files, deleted_files)
  }
}
