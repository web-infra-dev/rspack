use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::{
  ApplyContext, CompilerOptions, Content, ModuleFactoryCreateData,
  NormalModuleFactoryResolveForScheme, NormalModuleReadResource, Plugin, PluginContext,
  ResourceData,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

static URI_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"(?i)^https?:\/\/[^\s/$.?#].[^\s]*$").expect("Invalid Regex"));

#[plugin]
#[derive(Debug, Default)]
pub struct HttpUriPlugin;

#[plugin_hook(NormalModuleFactoryResolveForScheme for HttpUriPlugin)]
async fn resolve_for_scheme(
  &self,
  _data: &mut ModuleFactoryCreateData,
  resource_data: &mut ResourceData,
) -> Result<Option<bool>> {
  if resource_data.get_scheme().is_http()
    && let Some(captures) = URI_REGEX.captures(&resource_data.resource)
  {
    dbg!(&captures);
    return Ok(None);
  }
  Ok(None)
}

#[plugin_hook(NormalModuleReadResource for HttpUriPlugin)]
async fn read_resource(&self, resource_data: &ResourceData) -> Result<Option<Content>> {
  if resource_data.get_scheme().is_http()
    && let Some(captures) = URI_REGEX.captures(&resource_data.resource)
  {
    dbg!(&captures);
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
