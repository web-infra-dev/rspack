use std::{path::PathBuf, sync::Arc};

use rspack_cacheable::{
  cacheable,
  with::{AsInner, AsString},
};

#[cacheable]
#[derive(Debug, PartialEq, Eq)]
struct Data {
  #[with(AsInner<AsString>)]
  path: Arc<PathBuf>,
  #[with(AsInner)]
  block1: once_cell::sync::OnceCell<usize>,
  #[with(AsInner)]
  block2: once_cell::sync::OnceCell<usize>,
}

#[test]
fn test_as_inner() {
  let data = Data {
    path: Arc::new(PathBuf::from("/abc")),
    block1: once_cell::sync::OnceCell::new(),
    block2: once_cell::sync::OnceCell::with_value(2),
  };

  let bytes = rspack_cacheable::to_bytes(&data, &()).unwrap();
  let new_data: Data = rspack_cacheable::from_bytes(&bytes, &()).unwrap();
  assert_eq!(data, new_data);
}
