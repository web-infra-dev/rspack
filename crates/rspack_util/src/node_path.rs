use rspack_paths::{Utf8Path, Utf8PathBuf};

pub trait NodePath {
  fn node_join(&self, path: impl AsRef<Utf8Path>) -> Utf8PathBuf;
}

impl NodePath for Utf8Path {
  // There are some differences between the node method in Node.js and the join method in Rust's Path
  // In Rust, when the join method is passed an absolute path, the result is the absolute path itself, similar to using cd with an absolute path in the command line
  // In Node.js, when the join method is passed an absolute path, it simply concatenates the paths and then normalizes the resulting path
  fn node_join(&self, path: impl AsRef<Utf8Path>) -> Utf8PathBuf {
    let path = path.as_ref();
    let path = if path.is_absolute() {
      #[allow(clippy::unwrap_used)]
      path.strip_prefix("/").unwrap()
    } else {
      path
    };
    self.join(path)
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_node_join() {
    assert_eq!(
      Utf8Path::new("foo").node_join("/bar"),
      Utf8PathBuf::from("foo/bar")
    );
  }
}
