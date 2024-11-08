use std::sync::Mutex;

use rustc_hash::FxHashMap as HashMap;

use super::Storage;

/// Memory Storage
///
/// This storage is used to write unit test cases.
/// Rspack will reuse previous compilation as memory cache.
#[derive(Debug, Default)]
pub struct MemoryStorage {
  inner: Mutex<HashMap<String, HashMap<Vec<u8>, Vec<u8>>>>,
}

impl Storage for MemoryStorage {
  fn get_all(&self, scope: &str) -> Vec<(Vec<u8>, Vec<u8>)> {
    if let Some(value) = self.inner.lock().unwrap().get(scope) {
      value.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    } else {
      vec![]
    }
  }
  fn set(&self, scope: &str, key: Vec<u8>, value: Vec<u8>) {
    let mut map = self.inner.lock().unwrap();
    let inner = map.entry(String::from(scope)).or_default();
    inner.insert(key, value);
  }
  fn remove(&self, scope: &str, key: &[u8]) {
    let mut map = self.inner.lock().unwrap();
    map.get_mut(scope).map(|map| map.remove(key));
  }
  fn idle(&self) {}
}

#[cfg(test)]
mod tests {
  use super::MemoryStorage;
  use crate::cache::persistent::storage::Storage;

  #[test]
  fn should_memory_storage_works() {
    let scope = "test";
    let storage = MemoryStorage::default();
    storage.set(scope, "a".as_bytes().to_vec(), "abc".as_bytes().to_vec());
    storage.set(scope, "b".as_bytes().to_vec(), "bcd".as_bytes().to_vec());

    let arr = storage.get_all(scope);
    assert_eq!(arr.len(), 2);
    for (key, value) in arr {
      if &key == "a".as_bytes() {
        assert_eq!(&value, "abc".as_bytes());
      } else {
        assert_eq!(&value, "bcd".as_bytes());
      }
    }

    storage.remove(scope, "b".as_bytes());
    let arr = storage.get_all(scope);
    assert_eq!(arr.len(), 1);
  }
}
