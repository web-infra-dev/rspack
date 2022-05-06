use std::path::Path;
use sugar_path::{self, PathSugar};
pub fn normalize_path(path: &str, root: &str) -> String {
  let res = Path::new(&root)
    .relative(Path::new(&path))
    .to_string_lossy()
    .to_string();

  res
}
