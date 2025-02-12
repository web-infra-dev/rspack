use std::sync::Arc;

use rspack_core::{
  ApplyContext, BoxLoader, CompilerOptions, Context, ModuleRuleUseLoader,
  NormalModuleFactoryResolveLoader, Plugin, PluginContext, Resolver,
};
use rspack_error::{
  miette::{miette, LabeledSpan, SourceOffset},
  Result,
};
use rspack_hook::{plugin, plugin_hook};

use crate::{config::Config, LIGHTNINGCSS_LOADER_IDENTIFIER};

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

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    ctx
      .context
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
    let config: crate::config::RawConfig = serde_json::from_str(options).map_err(|e| {
      serde_error_to_miette(
        e,
        options,
        "Could not parse builtin:lightningcss-loader options",
      )
    })?;
    // TODO: builtin-loader supports function
    return Ok(Some(Arc::new(crate::LightningCssLoader::new(
      None,
      Config::try_from(config)?,
      loader_request,
    ))));
  }

  Ok(None)
}

// convert serde_error to miette report for pretty error
pub fn serde_error_to_miette(
  e: serde_json::Error,
  content: &str,
  msg: &str,
) -> rspack_error::miette::Report {
  let offset = SourceOffset::from_location(content, e.line(), e.column());
  let span = LabeledSpan::at_offset(offset.offset(), e.to_string());
  miette!(labels = vec![span], "{msg}").with_source_code(content.to_owned())
}
