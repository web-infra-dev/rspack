use rustc_hash::FxHashMap as HashMap;

use crate::{Result, Storage};

/// Memory-based storage implementation
///
/// All data is stored in an in-memory HashMap with no persistence.
/// Mainly used for:
/// - Unit testing
#[derive(Debug, Default)]
pub struct MemoryStorage {
  /// Internal storage structure: scope -> (key -> value)
  #[allow(clippy::type_complexity)]
  inner: HashMap<String, HashMap<Vec<u8>, Vec<u8>>>,
}

#[async_trait::async_trait]
impl Storage for MemoryStorage {
  async fn load(&self, scope: &'static str) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
    if let Some(value) = self.inner.get(scope) {
      Ok(value.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
    } else {
      Ok(vec![])
    }
  }

  fn set(&mut self, scope: &'static str, key: Vec<u8>, value: Vec<u8>) {
    let inner = self.inner.entry(String::from(scope)).or_default();
    inner.insert(key, value);
  }

  fn remove(&mut self, scope: &'static str, key: &[u8]) {
    self.inner.get_mut(scope).map(|map| map.remove(key));
  }

  async fn save(&mut self) -> Result<()> {
    // MemoryStorage holds all data in memory; nothing to persist
    Ok(())
  }

  async fn flush(&self) {
    // MemoryStorage has no background tasks; nothing to flush
  }

  async fn reset(&mut self) {
    self.inner.clear();
  }

  async fn scopes(&self) -> Result<Vec<String>> {
    Ok(self.inner.keys().cloned().collect())
  }
}

#[cfg(test)]
mod tests {
  use super::{MemoryStorage, Storage};

  #[tokio::test]
  async fn should_memory_storage_works() {
    let scope = "test";
    let mut storage = MemoryStorage::default();
    storage.set(scope, "a".as_bytes().to_vec(), "abc".as_bytes().to_vec());
    storage.set(scope, "b".as_bytes().to_vec(), "bcd".as_bytes().to_vec());

    let arr = storage.load(scope).await.unwrap();
    assert_eq!(arr.len(), 2);
    for (key, value) in arr {
      if key == "a".as_bytes() {
        assert_eq!(value, "abc".as_bytes());
      } else {
        assert_eq!(value, "bcd".as_bytes());
      }
    }

    storage.remove(scope, "b".as_bytes());
    let arr = storage.load(scope).await.unwrap();
    assert_eq!(arr.len(), 1);
    storage.reset().await;
    let arr = storage.load(scope).await.unwrap();
    assert_eq!(arr.len(), 0);
  }
}
