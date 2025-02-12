use std::sync::Arc;

use rspack_core::{
  ApplyContext, BoxLoader, CompilerOptions, Context, ModuleRuleUseLoader,
  NormalModuleFactoryResolveLoader, Plugin, PluginContext, Resolver,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

use crate::PREACT_REFRESH_LOADER_IDENTIFIER;

#[plugin]
#[derive(Debug)]
pub struct PreactRefreshLoaderPlugin;

impl PreactRefreshLoaderPlugin {
  pub fn new() -> Self {
    Self::new_inner()
  }
}

impl Default for PreactRefreshLoaderPlugin {
  fn default() -> Self {
    Self::new()
  }
}

impl Plugin for PreactRefreshLoaderPlugin {
  fn name(&self) -> &'static str {
    "PreactRefreshLoaderPlugin"
  }

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    ctx
      .context
      .normal_module_factory_hooks
      .resolve_loader
      .tap(resolve_loader::new(self));
    Ok(())
  }
}

#[plugin_hook(NormalModuleFactoryResolveLoader for PreactRefreshLoaderPlugin)]
pub(crate) async fn resolve_loader(
  &self,
  _context: &Context,
  _resolver: &Resolver,
  l: &ModuleRuleUseLoader,
) -> Result<Option<BoxLoader>> {
  let loader_request = &l.loader;

  if loader_request.starts_with(PREACT_REFRESH_LOADER_IDENTIFIER) {
    return Ok(Some(Arc::new(
      crate::PreactRefreshLoader::default().with_identifier(loader_request.as_str().into()),
    )));
  }

  Ok(None)
}
