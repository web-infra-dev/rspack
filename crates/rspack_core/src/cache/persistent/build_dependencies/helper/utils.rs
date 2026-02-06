use std::path::{Component, Path};

/// Check if a path is a node package path.
pub(crate) fn is_node_package_path(path: &Path) -> bool {
  let mut result = false;
  for comp in path.components() {
    if let Component::Normal(os_str) = comp {
      let comp_str = os_str.to_string_lossy();
      if comp_str == "node_modules" {
        result = true;
      }
      if comp_str.starts_with('.') {
        result = false;
      }
    }
  }
  result
}

#[cfg(test)]
mod test {
  use std::path::PathBuf;

  use rspack_paths::ArcPath;

  use super::is_node_package_path;

  fn generate_arc_path(path: &str) -> ArcPath {
    let path_buf = PathBuf::from(path);
    ArcPath::from(path_buf)
  }

  #[test]
  fn check_is_node_package() {
    assert!(!is_node_package_path(&generate_arc_path(
      "/root/a/index.js"
    )),);
    assert!(is_node_package_path(&generate_arc_path(
      "/root/node_modules/a/index.js"
    )),);
    assert!(!is_node_package_path(&generate_arc_path(
      "/root/node_modules/.a/index.js"
    )),);
    assert!(is_node_package_path(&generate_arc_path(
      "/root/node_modules/.a/node_modules/a/index.js"
    )),);
  }
}
