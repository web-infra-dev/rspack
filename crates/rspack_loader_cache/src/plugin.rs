use std::sync::Arc;

use rspack_core::{
  BoxLoader, Context, ModuleRuleUseLoader, NormalModuleFactoryResolveLoader, Plugin, Resolver,
};
use rspack_error::{Result, SerdeResultToRspackResultExt};
use rspack_hook::{plugin, plugin_hook};

use crate::{CACHE_LOADER_IDENTIFIER, CacheLoader, CacheLoaderOptions};

#[plugin]
#[derive(Debug)]
pub struct CacheLoaderPlugin;

impl CacheLoaderPlugin {
  pub fn new() -> Self {
    Self::new_inner()
  }
}

impl Default for CacheLoaderPlugin {
  fn default() -> Self {
    Self::new()
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

  let raw_options = loader.options.as_deref().unwrap_or("{}");
  let options: CacheLoaderOptions = serde_json::from_str(raw_options)
    .to_rspack_result_with_detail(raw_options, "Failed to parse builtin:cache-loader options")?;

  Ok(Some(Arc::new(CacheLoader::new(
    loader.loader.as_str().into(),
    options,
  ))))
}
