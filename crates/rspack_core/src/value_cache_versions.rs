use std::sync::Arc;

use rustc_hash::FxHashMap as HashMap;

#[derive(Debug, Default, Clone)]
pub struct ValueCacheVersions(Arc<HashMap<String, String>>);

impl ValueCacheVersions {
  pub fn get(&self, key: &str) -> Option<&String> {
    self.0.get(key)
  }

  pub fn insert(&mut self, key: String, value: String) {
    Arc::get_mut(&mut self.0)
      .expect("value cache versions must be mutated before being shared")
      .insert(key, value);
  }

  pub fn has_diff(&self, value_dependencies: &HashMap<String, String>) -> bool {
    for (key, value) in value_dependencies {
      let Some(current) = self.get(key) else {
        return true;
      };
      if value != current {
        return true;
      }
    }
    false
  }
}
