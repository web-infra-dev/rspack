use rustc_hash::FxHashMap as HashMap;

#[derive(Debug, Default, Clone)]
pub struct ValueCacheVersions {
  values: HashMap<String, String>,
  defaults: HashMap<String, String>,
}

impl ValueCacheVersions {
  pub fn get(&self, key: &str) -> Option<&String> {
    self.values.get(key).or_else(|| {
      self
        .defaults
        .iter()
        .filter(|(prefix, _)| key.starts_with(prefix.as_str()))
        .max_by_key(|(prefix, _)| prefix.len())
        .map(|(_, value)| value)
    })
  }

  pub fn insert(&mut self, key: String, value: String) {
    self.values.insert(key, value);
  }

  /// A prefix default lets plugins validate values for inputs absent from their
  /// current set, including inputs restored from a previous compiler's cache.
  pub fn insert_default(&mut self, prefix: String, value: String) {
    self.defaults.insert(prefix, value);
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
