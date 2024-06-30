use anyhow::Context;
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::Client; // Add reqwest for HTTP requests
use rspack_core::{
  ApplyContext, CompilerOptions, Content, ModuleFactoryCreateData,
  NormalModuleFactoryResolveForScheme, NormalModuleReadResource, Plugin, PluginContext,
  ResourceData,
};
use rspack_error::error;
use rspack_error::{AnyhowError, Error, Result};
use rspack_hook::{plugin, plugin_hook}; // Add this import
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
    let client = Client::new();
    let response = client
      .get(&resource_data.resource)
      .send()
      .await
      .context("Failed to send HTTP request")
      .map_err(|err| {
        error!(err.to_string());
        AnyhowError::from(err) // Convert to AnyhowError which implements Diagnostic
      })?; // Use `into()` to convert anyhow::Error to rspack_error::Error
    let content = response
      .bytes()
      .await
      .context("Failed to read response bytes")
      .map_err(|err| {
        error!(err.to_string());
        AnyhowError::from(err) // Convert to AnyhowError which implements Diagnostic
      })?;
    dbg!("Response body: {:?}", &content); // Log the response body
    return Ok(Some(Content::Buffer(content.to_vec())));
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
