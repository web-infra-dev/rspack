#[derive(Debug)]
pub struct CacheableContext;

impl rspack_cacheable::CacheableContext for CacheableContext {
  fn project_root(&self) -> Option<&std::path::Path> {
    None
  }
}
