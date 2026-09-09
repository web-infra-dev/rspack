use super::{CacheFacade, FileSystemInfo};

/// Resolution cache scoped to one compilation. Cloning shares only cache and
/// filesystem-info handles; it never changes a shared resolver factory's state.
#[derive(Debug, Clone)]
pub struct ResolverCache {
  pub(crate) cache: CacheFacade,
  pub(crate) file_system_info: FileSystemInfo,
}

impl ResolverCache {
  pub(crate) fn new(cache: CacheFacade, file_system_info: FileSystemInfo) -> Self {
    Self {
      cache,
      file_system_info,
    }
  }

  pub(crate) fn child(&self, name: &str) -> Self {
    Self {
      cache: self.cache.get_child_cache(name),
      file_system_info: self.file_system_info.clone(),
    }
  }
}
