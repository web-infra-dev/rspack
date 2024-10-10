use std::{path::PathBuf, sync::Arc};

use rspack_cacheable::{
  cacheable,
  with::{AsInner, AsString},
};

#[cacheable]
#[derive(Debug, PartialEq, Eq)]
struct Data {
  #[cacheable(with=AsInner<AsString>)]
  block1: once_cell::sync::OnceCell<PathBuf>,
  #[cacheable(with=AsInner)]
  block2: once_cell::sync::OnceCell<usize>,
  #[cacheable(with=AsInner)]
  block3: Arc<usize>,
}

#[test]
fn test_as_inner() {
  let data = Data {
    block1: once_cell::sync::OnceCell::with_value(PathBuf::from("/abc")),
    block2: once_cell::sync::OnceCell::with_value(1),
    block3: Arc::new(2),
  };

  let bytes = rspack_cacheable::to_bytes(&data, &()).unwrap();
  let new_data: Data = rspack_cacheable::from_bytes(&bytes, &()).unwrap();
  assert_eq!(data, new_data);
}
