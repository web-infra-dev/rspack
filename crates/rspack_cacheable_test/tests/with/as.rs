use std::{any::Any, path::PathBuf};

use rspack_cacheable::{
  Error, enable_cacheable as cacheable,
  with::{As, AsConverter},
};

#[derive(Debug, PartialEq, Eq)]
struct UnCacheableData(PathBuf);

#[cacheable]
struct CacheableData(String);

impl AsConverter<UnCacheableData> for CacheableData {
  fn serialize(data: &UnCacheableData, _ctx: &dyn Any) -> Result<Self, Error> {
    Ok(Self(data.0.to_string_lossy().to_string()))
  }
  fn deserialize(self, _ctx: &dyn Any) -> Result<UnCacheableData, Error> {
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
