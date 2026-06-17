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

  pub fn has_diff<'a>(
    &self,
    value_dependencies: impl IntoIterator<Item = (&'a String, &'a String)>,
  ) -> bool {
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
