use std::sync::Arc;

use rspack_core::{
  ApplyContext, BoxLoader, CompilerOptions, Context, ModuleRuleUseLoader,
  NormalModuleFactoryAfterResolveLoader, Plugin, PluginContext, Resolver,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

use crate::REACT_REFRESH_LOADER_IDENTIFIER;

#[plugin]
#[derive(Debug)]
pub struct ReactRefreshLoaderPlugin;

impl ReactRefreshLoaderPlugin {
  pub fn new() -> Self {
    Self::new_inner()
  }
}

impl Default for ReactRefreshLoaderPlugin {
  fn default() -> Self {
    Self::new()
  }
}

impl Plugin for ReactRefreshLoaderPlugin {
  fn name(&self) -> &'static str {
    "ReactRefreshLoaderPlugin"
  }

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    ctx
      .context
      .normal_module_factory_hooks
      .after_resolve_loader
      .tap(after_resolve_loader::new(self));

    Ok(())
  }
}

#[plugin_hook(NormalModuleFactoryAfterResolveLoader for ReactRefreshLoaderPlugin)]
pub(crate) async fn after_resolve_loader(
  &self,
  _context: &Context,
  _resolver: &Resolver,
  l: &ModuleRuleUseLoader,
) -> Result<Option<BoxLoader>> {
  let loader_request = &l.loader;

  if loader_request.starts_with(REACT_REFRESH_LOADER_IDENTIFIER) {
    println!(
      "[ReactRefreshLoaderPlugin] Adding React Refresh Loader for: {}",
      loader_request
    );
    return Ok(Some(Arc::new(
      crate::ReactRefreshLoader::default().with_identifier(loader_request.as_str().into()),
    )));
  }

  Ok(None)
}
