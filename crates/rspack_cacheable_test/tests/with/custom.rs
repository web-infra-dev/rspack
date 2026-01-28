use std::path::PathBuf;

use rspack_cacheable::{
  ContextGuard, Result, enable_cacheable as cacheable,
  with::{Custom, CustomConverter},
};

#[cacheable(with=Custom)]
#[derive(Debug, PartialEq, Eq)]
struct Data(PathBuf);

impl CustomConverter for Data {
  type Target = String;
  fn serialize(&self, _guard: &ContextGuard) -> Result<Self::Target> {
    Ok(self.0.to_string_lossy().to_string())
  }
  fn deserialize(data: Self::Target, _guard: &ContextGuard) -> Result<Self> {
    Ok(Data(PathBuf::from(&data)))
  }
}

#[test]
fn test_custom() {
  let data = Data(PathBuf::from("/home/user"));

  let bytes = rspack_cacheable::to_bytes(&data, &()).unwrap();
  let new_data: Data = rspack_cacheable::from_bytes(&bytes, &()).unwrap();
  assert_eq!(data, new_data);
}
