use std::{any::Any, path::PathBuf};

use rspack_cacheable::{
  DeserializeError, SerializeError, enable_cacheable as cacheable,
  with::{Custom, CustomConverter},
};

#[cacheable(with=Custom)]
#[derive(Debug, PartialEq, Eq)]
struct Data(PathBuf);

impl CustomConverter for Data {
  type Target = String;
  fn serialize(&self, _ctx: &dyn Any) -> Result<Self::Target, SerializeError> {
    Ok(self.0.to_string_lossy().to_string())
  }
  fn deserialize(data: Self::Target, _ctx: &dyn Any) -> Result<Self, DeserializeError> {
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
