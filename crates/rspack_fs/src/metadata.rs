use std::fs::Metadata;

pub struct FSMetadata {
  pub is_file: bool,
  pub is_dir: bool,
  pub is_symlink: bool,
}

impl FSMetadata {
  pub fn new(is_file: bool, is_dir: bool, is_symlink: bool) -> Self {
    Self {
      is_file,
      is_dir,
      is_symlink,
    }
  }
}

impl From<Metadata> for FSMetadata {
  fn from(value: Metadata) -> Self {
    Self {
      is_file: value.is_file(),
      is_dir: value.is_dir(),
      is_symlink: value.is_symlink(),
    }
  }
}
