use napi_derive::napi;
use rspack_fs::FSMetadata;

#[napi(object, js_name = "NodeFSMetadata")]
pub struct NodeFSMetadata {
  pub is_file: bool,
  pub is_dir: bool,
  pub is_symlink: bool,
}

impl From<NodeFSMetadata> for FSMetadata {
  fn from(value: NodeFSMetadata) -> Self {
    Self::new(value.is_file, value.is_dir, value.is_symlink)
  }
}
