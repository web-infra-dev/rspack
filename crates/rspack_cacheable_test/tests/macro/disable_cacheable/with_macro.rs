use std::path::PathBuf;

use rspack_cacheable::{disable_cacheable, utils::PortablePath, with::As};

#[disable_cacheable]
struct FileInfo {
  #[cacheable(with=As<PortablePath>)]
  path: PathBuf,
}

#[test]
fn with_macro() {
  let data = FileInfo {
    path: PathBuf::default(),
  };
  assert_eq!(data.path, PathBuf::default())
}
