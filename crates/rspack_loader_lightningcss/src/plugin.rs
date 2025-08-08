use std::sync::Arc;

use rspack_core::{
  BoxLoader, Context, ModuleRuleUseLoader, NormalModuleFactoryResolveLoader, Plugin, Resolver,
};
use rspack_error::{Result, SerdeResultToRspackResultExt};
use rspack_hook::{plugin, plugin_hook};

use crate::{LIGHTNINGCSS_LOADER_IDENTIFIER, config::Config};

#[plugin]
#[derive(Debug)]
pub struct LightningcssLoaderPlugin;

impl LightningcssLoaderPlugin {
  pub fn new() -> Self {
    Self::new_inner()
  }
}

impl Default for LightningcssLoaderPlugin {
  fn default() -> Self {
    Self::new()
  }
}

impl Plugin for LightningcssLoaderPlugin {
  fn name(&self) -> &'static str {
    "LightningcssLoaderPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .normal_module_factory_hooks
      .resolve_loader
      .tap(resolve_loader::new(self));
    Ok(())
  }
}

#[plugin_hook(NormalModuleFactoryResolveLoader for LightningcssLoaderPlugin)]
pub(crate) async fn resolve_loader(
  &self,
  _context: &Context,
  _resolver: &Resolver,
  l: &ModuleRuleUseLoader,
) -> Result<Option<BoxLoader>> {
  let loader_request = &l.loader;
  let options = l.options.as_deref().unwrap_or("{}");

  if loader_request.starts_with(LIGHTNINGCSS_LOADER_IDENTIFIER) {
    let config: crate::config::RawConfig = serde_json::from_str(options)
      .to_rspack_result_with_detail(
        options,
        "Could not parse builtin:lightningcss-loader options",
      )?;
    // TODO: builtin-loader supports function
    return Ok(Some(Arc::new(crate::LightningCssLoader::new(
      None,
      Config::try_from(config)?,
      loader_request,
    ))));
  }

  Ok(None)
}
