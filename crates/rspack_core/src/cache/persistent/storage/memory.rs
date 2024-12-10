use std::sync::{Arc, Mutex};

use rspack_error::Result;
use rspack_storage::Storage;
use rustc_hash::FxHashMap as HashMap;
use tokio::sync::oneshot::{channel, Receiver};

/// Memory Storage
///
/// This storage is used to write unit test cases.
/// Rspack will reuse previous compilation as memory cache.
#[derive(Debug, Default)]
pub struct MemoryStorage {
  #[allow(clippy::type_complexity)]
  inner: Mutex<HashMap<String, HashMap<Vec<u8>, Vec<u8>>>>,
}

#[async_trait::async_trait]
impl Storage for MemoryStorage {
  async fn load(&self, scope: &'static str) -> Result<Vec<(Arc<Vec<u8>>, Arc<Vec<u8>>)>> {
    if let Some(value) = self.inner.lock().expect("should get lock").get(scope) {
      Ok(
        value
          .iter()
          .map(|(k, v)| (Arc::new(k.clone()), Arc::new(v.clone())))
          .collect(),
      )
    } else {
      Ok(vec![])
    }
  }
  fn set(&self, scope: &str, key: Vec<u8>, value: Vec<u8>) {
    let mut map = self.inner.lock().expect("should get lock");
    let inner = map.entry(String::from(scope)).or_default();
    inner.insert(key, value);
  }
  fn remove(&self, scope: &str, key: &[u8]) {
    let mut map = self.inner.lock().expect("should get lock");
    map.get_mut(scope).map(|map| map.remove(key));
  }
  fn trigger_save(&self) -> Result<Receiver<Result<()>>> {
    let (rs, rx) = channel::<Result<()>>();
    rs.send(Ok(()));
    Ok(rx)
  }
}

/*#[cfg(test)]
mod tests {
  use rspack_storage::Storage;

  use super::MemoryStorage;

  #[test]
  fn should_memory_storage_works() {
    let scope = "test";
    let storage = MemoryStorage::default();
    storage.set(scope, "a".as_bytes().to_vec(), "abc".as_bytes().to_vec());
    storage.set(scope, "b".as_bytes().to_vec(), "bcd".as_bytes().to_vec());

    let arr = storage.load(scope);
    assert_eq!(arr.len(), 2);
    for (key, value) in arr {
      if key == "a".as_bytes() {
        assert_eq!(&value, "abc".as_bytes());
      } else {
        assert_eq!(&value, "bcd".as_bytes());
      }
    }

    storage.remove(scope, "b".as_bytes());
    let arr = storage.load(scope);
    assert_eq!(arr.len(), 1);
  }
}
*/
