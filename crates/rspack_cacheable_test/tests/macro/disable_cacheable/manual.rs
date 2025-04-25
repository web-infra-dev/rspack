use std::path::PathBuf;

use rspack_cacheable::with::AsString;

struct FileInfo {
  path: PathBuf,
}

fn _fileinfo() {
  // use with type to avoid clippy unused import
  let _: Option<(AsString,)> = None;
}

#[test]
fn manual() {
  let data = FileInfo {
    path: PathBuf::default(),
  };
  assert_eq!(data.path, PathBuf::default())
}
