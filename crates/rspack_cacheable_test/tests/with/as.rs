use std::path::PathBuf;

use rspack_cacheable::{
  ContextGuard, Result, enable_cacheable as cacheable,
  with::{As, AsConverter},
};

#[derive(Debug, PartialEq, Eq)]
struct UnCacheableData(PathBuf);

#[cacheable]
struct CacheableData(String);

impl AsConverter<UnCacheableData> for CacheableData {
  fn serialize(data: &UnCacheableData, _guard: &ContextGuard) -> Result<Self> {
    Ok(Self(data.0.to_string_lossy().to_string()))
  }
  fn deserialize(self, _guard: &ContextGuard) -> Result<UnCacheableData> {
    Ok(UnCacheableData(PathBuf::from(&self.0)))
  }
}

#[cacheable]
#[derive(Debug, PartialEq, Eq)]
struct Data {
  #[cacheable(with=As<CacheableData>)]
  inner: UnCacheableData,
}

#[test]
fn test_as() {
  let data = Data {
    inner: UnCacheableData(PathBuf::from("/home/user")),
  };

  let bytes = rspack_cacheable::to_bytes(&data, &()).unwrap();
  let new_data: Data = rspack_cacheable::from_bytes(&bytes, &()).unwrap();
  assert_eq!(data, new_data);
}
