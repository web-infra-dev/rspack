use rspack_paths::Utf8PathBuf;

#[derive(Debug)]
pub struct CacheableContext {
  pub project_path: Utf8PathBuf,
}

impl rspack_cacheable::CacheableContext for CacheableContext {
  fn project_root(&self) -> Option<&std::path::Path> {
    Some(self.project_path.as_std_path())
  }
}
