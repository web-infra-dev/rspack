use rustc_hash::FxHashMap as HashMap;

#[derive(Debug, Default, Clone)]
pub struct ValueCacheVersions {
  values: HashMap<String, String>,
  defaults: HashMap<String, String>,
}

impl ValueCacheVersions {
  pub fn get(&self, key: &str) -> Option<&String> {
    self.values.get(key)
  }

  pub fn insert(&mut self, key: String, value: String) {
    self.values.insert(key, value);
  }

  /// Used only by `has_diff` for dependencies without an explicit current value.
  /// This covers inputs restored from cache but absent from a plugin's current set.
  pub fn insert_default_for_prefix(&mut self, prefix: String, value: String) {
    self.defaults.insert(prefix, value);
  }

  pub fn has_diff(&self, value_dependencies: &HashMap<String, String>) -> bool {
    for (key, value) in value_dependencies {
      let Some(current) = self.get(key).or_else(|| {
        self
          .defaults
          .iter()
          .filter(|(prefix, _)| key.starts_with(prefix.as_str()))
          .max_by_key(|(prefix, _)| prefix.len())
          .map(|(_, value)| value)
      }) else {
        return true;
      };
      if value != current {
        return true;
      }
    }
    false
  }
}
