use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::{
  ApplyContext, CompilerOptions, Content, ModuleFactoryCreateData,
  NormalModuleFactoryResolveForScheme, NormalModuleReadResource, Plugin, PluginContext,
  ResourceData,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

static EXTERNAL_HTTP_REQUEST: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^(//|https?://|#)").expect("Invalid regex"));
static EXTERNAL_HTTP_STD_REQUEST: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^(//|https?://|std:)").expect("Invalid regex"));
static EXTERNAL_CSS_REQUEST: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^\.css(\?|$)").expect("Invalid regex"));

#[plugin]
#[derive(Debug, Default)]
pub struct HttpUriPlugin;

#[plugin_hook(NormalModuleFactoryResolveForScheme for HttpUriPlugin)]
async fn resolve_for_scheme(
  &self,
  _data: &mut ModuleFactoryCreateData,
  resource_data: &mut ResourceData,
) -> Result<Option<bool>> {
  if resource_data.get_scheme().is_http() && EXTERNAL_HTTP_REQUEST.is_match(&resource_data.resource)
  {
    dbg!(&resource_data.resource);
    return Ok(None);
  }
  Ok(None)
}

#[plugin_hook(NormalModuleReadResource for HttpUriPlugin)]
async fn read_resource(&self, resource_data: &ResourceData) -> Result<Option<Content>> {
  dbg!("reading resource");
  if resource_data.get_scheme().is_http() && EXTERNAL_HTTP_REQUEST.is_match(&resource_data.resource)
  {
    dbg!(&resource_data.resource);
    // Implement your logic for reading HTTP resources here
  }
  Ok(None)
}

#[async_trait::async_trait]
impl Plugin for HttpUriPlugin {
  fn name(&self) -> &'static str {
    "rspack.HttpUriPlugin"
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &mut CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .normal_module_factory_hooks
      .resolve_for_scheme
      .tap(resolve_for_scheme::new(self));
    ctx
      .context
      .normal_module_hooks
      .read_resource
      .tap(read_resource::new(self));
    Ok(())
  }
}
