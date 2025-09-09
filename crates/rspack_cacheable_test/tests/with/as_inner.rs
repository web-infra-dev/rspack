use std::{
  path::PathBuf,
  sync::{Arc, OnceLock},
};

use rspack_cacheable::{
  enable_cacheable as cacheable,
  with::{AsInner, AsString},
};

#[cacheable]
#[derive(Debug, PartialEq, Eq)]
struct Data {
  #[cacheable(with=AsInner<AsString>)]
  block1: OnceLock<PathBuf>,
  #[cacheable(with=AsInner)]
  block2: OnceLock<usize>,
  #[cacheable(with=AsInner)]
  block3: Arc<usize>,
}

#[test]
fn test_as_inner() {
  let data = Data {
    block1: OnceLock::from(PathBuf::from("/abc")),
    block2: OnceLock::from(1),
    block3: Arc::new(2),
  };

  let bytes = rspack_cacheable::to_bytes(&data, &()).unwrap();
  let new_data: Data = rspack_cacheable::from_bytes(&bytes, &()).unwrap();
  assert_eq!(data, new_data);
}
