use std::{fs, path::PathBuf, time::UNIX_EPOCH};

use rspack_cacheable::cacheable;
use rustc_hash::FxHashMap as HashMap;

#[cacheable]
#[derive(Debug, PartialEq)]
pub enum Strategy {
  LibVersion(String),
  CompileTime(u64),
}

pub enum ValidateResult {
  Deleted,
  Modified,
  NoChange,
}

#[derive(Default)]
pub struct StrategyHelper {
  lib_version_cache: HashMap<PathBuf, Option<String>>,
}

impl StrategyHelper {
  pub fn lib_version(&mut self, path: &PathBuf) -> Option<String> {
    if let Some(version) = self.lib_version_cache.get(path) {
      return version.clone();
    }

    // 1. try get package.json version in current path
    let res = package_json_version(path).or_else(|| {
      // 2. try get lib version in parent path
      // 3. if parent path is none, return none
      path
        .parent()
        .and_then(|parent| self.lib_version(&parent.to_path_buf()))
    });
    self.lib_version_cache.insert(path.clone(), res.clone());
    res
  }

  pub fn validate(&mut self, path: &PathBuf, strategy: &Strategy) -> ValidateResult {
    match strategy {
      Strategy::LibVersion(version) => {
        if let Some(ref cur_version) = self.lib_version(path) {
          if cur_version == version {
            ValidateResult::NoChange
          } else {
            ValidateResult::Modified
          }
        } else {
          ValidateResult::Deleted
        }
      }
      Strategy::CompileTime(compile_time) => {
        if let Some(ref modified_time) = modified_time(path) {
          if modified_time > compile_time {
            ValidateResult::Modified
          } else {
            ValidateResult::NoChange
          }
        } else {
          ValidateResult::Deleted
        }
      }
    }
  }
}

fn modified_time(path: &PathBuf) -> Option<u64> {
  if let Ok(info) = fs::metadata(path) {
    if let Ok(time) = info.modified() {
      if let Ok(s) = time.duration_since(UNIX_EPOCH) {
        return Some(s.as_secs());
      }
    }
  }
  None
}

fn package_json_version(path: &PathBuf) -> Option<String> {
  if let Ok(content) = fs::read(path.join("package.json")) {
    if let Ok(package_json) =
      serde_json::from_slice::<serde_json::Map<String, serde_json::Value>>(&content)
    {
      if let Some(serde_json::Value::String(version)) = package_json.get("version") {
        return Some(version.clone());
      }
    }
  }
  None
}
