use std::path::PathBuf;

use rspack_cacheable::{disable_cacheable, with::AsString};

#[disable_cacheable]
struct FileInfo {
  #[cacheable(with=AsString)]
  path: PathBuf,
}

#[test]
fn with_macro() {
  let data = FileInfo {
    path: PathBuf::default(),
  };
  assert_eq!(data.path, PathBuf::default())
}
