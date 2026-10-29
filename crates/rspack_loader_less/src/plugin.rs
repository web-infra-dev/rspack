use std::sync::Arc;

use rspack_core::{
  BoxLoader, Context, ModuleFactoryCreateData, ModuleRuleUseLoader,
  NormalModuleFactoryResolveLoader, Plugin, Resolver,
};
use rspack_error::{Result, SerdeResultToRspackResultExt};
use rspack_hook::{plugin, plugin_hook};

use crate::{LESS_LOADER_IDENTIFIER, LessLoader};

#[plugin]
#[derive(Debug)]
pub struct LessLoaderPlugin;

impl LessLoaderPlugin {
  pub fn new() -> Self {
    Self::new_inner()
  }
}

impl Default for LessLoaderPlugin {
  fn default() -> Self {
    Self::new()
  }
}

impl Plugin for LessLoaderPlugin {
  fn name(&self) -> &'static str {
    "LessLoaderPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .normal_module_factory_hooks
      .resolve_loader
      .tap(resolve_loader::new(self));
    Ok(())
  }
}

#[plugin_hook(NormalModuleFactoryResolveLoader for LessLoaderPlugin)]
async fn resolve_loader(
  &self,
  _context: &Context,
  _resolver: &Resolver,
  loader: &ModuleRuleUseLoader,
  _data: &mut ModuleFactoryCreateData,
) -> Result<Option<BoxLoader>> {
  let request = &loader.loader;
  if request.split(['?', '#']).next() != Some(LESS_LOADER_IDENTIFIER) {
    return Ok(None);
  }
  let options = loader.options.as_deref().unwrap_or("{}");
  let loader = LessLoader::new(options)
    .to_rspack_result_with_detail(options, "Could not parse builtin:less-loader options")?
    .with_identifier(request.as_str().into());
  Ok(Some(Arc::new(loader)))
}
