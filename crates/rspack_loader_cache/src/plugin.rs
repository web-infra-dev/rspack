use std::sync::Arc;

use rspack_core::{
  BoxLoader, Context, ModuleRuleUseLoader, NormalModuleFactoryResolveLoader, Plugin, Resolver,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

use crate::{CACHE_LOADER_IDENTIFIER, CacheLoader, create_cache, remove_cache};

#[plugin]
#[derive(Debug)]
pub struct CacheLoaderPlugin {
  cache_id: u64,
}

impl CacheLoaderPlugin {
  pub fn new() -> Self {
    Self::new_inner(create_cache())
  }
}

impl Default for CacheLoaderPlugin {
  fn default() -> Self {
    Self::new()
  }
}

impl Drop for CacheLoaderPluginInner {
  fn drop(&mut self) {
    remove_cache(self.cache_id);
  }
}

impl Plugin for CacheLoaderPlugin {
  fn name(&self) -> &'static str {
    "CacheLoaderPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .normal_module_factory_hooks
      .resolve_loader
      .tap(resolve_loader::new(self));
    Ok(())
  }
}

#[plugin_hook(NormalModuleFactoryResolveLoader for CacheLoaderPlugin)]
pub(crate) async fn resolve_loader(
  &self,
  _context: &Context,
  _resolver: &Resolver,
  loader: &ModuleRuleUseLoader,
) -> Result<Option<BoxLoader>> {
  if !loader.loader.starts_with(CACHE_LOADER_IDENTIFIER) {
    return Ok(None);
  }

  Ok(Some(Arc::new(CacheLoader::new(
    loader.loader.as_str().into(),
    self.cache_id,
  ))))
}
