use rspack_cacheable::cacheable;
use rspack_tasks::fetch_new_dependency_id;
use serde::{Serialize, Serializer};
use slotmap::{Key, KeyData};

#[cacheable(hashable)]
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
#[repr(transparent)]
pub struct DependencyId(u64);

impl DependencyId {
  pub fn new() -> Self {
    Self::from(fetch_new_dependency_id())
  }

  pub fn as_u32(&self) -> u32 {
    self.0 as u32
  }

  pub fn as_ffi(&self) -> u64 {
    self.0
  }

  pub fn is_null(&self) -> bool {
    <Self as Key>::is_null(self)
  }
}

impl Default for DependencyId {
  fn default() -> Self {
    <Self as Key>::null()
  }
}

impl From<u32> for DependencyId {
  fn from(id: u32) -> Self {
    KeyData::from_ffi((1_u64 << 32) | u64::from(id)).into()
  }
}

impl From<KeyData> for DependencyId {
  fn from(value: KeyData) -> Self {
    Self(value.as_ffi())
  }
}

unsafe impl Key for DependencyId {
  fn data(&self) -> KeyData {
    KeyData::from_ffi(self.0)
  }
}

impl std::fmt::Debug for DependencyId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.data().fmt(f)
  }
}

impl std::hash::Hash for DependencyId {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    state.write_u32(self.as_u32());
  }
}

impl Serialize for DependencyId {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_u32(self.as_u32())
  }
}

#[cfg(test)]
mod tests {
  use rspack_tasks::within_compiler_context_for_testing_sync;
  use slotmap::SecondaryMap;

  use super::DependencyId;

  #[test]
  fn default_dependency_id_is_null_key() {
    assert!(DependencyId::default().is_null());
  }

  #[test]
  fn generated_dependency_ids_work_with_secondary_map() {
    within_compiler_context_for_testing_sync(|| {
      let first = DependencyId::new();
      let second = DependencyId::new();
      let mut map = SecondaryMap::new();

      assert_ne!(first, second);
      assert!(!first.is_null());
      assert!(!second.is_null());

      map.insert(first, "first");
      map.insert(second, "second");

      assert_eq!(map.get(first), Some(&"first"));
      assert_eq!(map.get(second), Some(&"second"));
    });
  }
}
