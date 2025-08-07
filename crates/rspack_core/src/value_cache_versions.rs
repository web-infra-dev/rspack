use rustc_hash::FxHashMap as HashMap;

#[derive(Debug, Default, Clone)]
pub struct ValueCacheVersions(HashMap<String, String>);

impl ValueCacheVersions {
  pub fn get(&self, key: &str) -> Option<&String> {
    self.0.get(key)
  }

  pub fn insert(&mut self, key: String, value: String) {
    self.0.insert(key, value);
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

  pub fn extend(&mut self, other: Self) {
    self.0.extend(other.0);
  }
}
