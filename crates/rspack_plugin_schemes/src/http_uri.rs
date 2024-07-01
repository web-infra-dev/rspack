use std::fs;
use std::path::Path;

use anyhow::Context;
use reqwest::Client;
use rspack_core::{
  ApplyContext, CompilerOptions, Content, ModuleFactoryCreateData,
  NormalModuleFactoryResolveForScheme, NormalModuleReadResource, Plugin, PluginContext,
  ResourceData,
};
use rspack_error::error;
use rspack_error::{AnyhowError, Result};
use rspack_hook::{plugin, plugin_hook};
use url::Url;

static HTTP_CACHE_DIR: &str = "http_cache";

#[plugin]
#[derive(Debug, Default)]
pub struct HttpUriPlugin;

#[plugin_hook(NormalModuleFactoryResolveForScheme for HttpUriPlugin)]
async fn resolve_for_scheme(
  &self,
  _data: &mut ModuleFactoryCreateData,
  resource_data: &mut ResourceData,
) -> Result<Option<bool>> {
  if resource_data.get_scheme().is_http() {
    let url = Url::parse(&resource_data.resource)
      .context("Invalid URL")
      .map_err(|err| {
        error!(err.to_string());
        AnyhowError::from(err)
      })?;

    let context_url = match Url::parse(&resource_data.resource) {
      Ok(url) => url.origin().ascii_serialization(),
      Err(_) => String::default(),
    };

    resource_data.context = Some(context_url);

    resource_data.resource = url.as_str().to_string();
    resource_data.resource_query = url.query().map(|q| q.to_string());
    resource_data.resource_fragment = url.fragment().map(|f| f.to_string());
    // resource_data.mimetype = Some(content_type);

    return Ok(Some(true));
  }
  Ok(None)
}

#[plugin_hook(NormalModuleReadResource for HttpUriPlugin)]
async fn read_resource(&self, resource_data: &ResourceData) -> Result<Option<Content>> {
  dbg!("reading resource");
  if resource_data.get_scheme().is_http() {
    dbg!(&resource_data.resource);

    // Check cache first
    let cache_path = format!(
      "{}/{}",
      HTTP_CACHE_DIR,
      resource_data.resource.replace("/", "_")
    );
    if Path::new(&cache_path).exists() {
      dbg!("Cache hit");
      let cached_content = fs::read(&cache_path)
        .context("Failed to read cached content")
        .map_err(|err| {
          error!(err.to_string());
          AnyhowError::from(err)
        })?;
      return Ok(Some(Content::Buffer(cached_content)));
    }

    let client = Client::new();
    let response = client
      .get(&resource_data.resource)
      .send()
      .await
      .context("Failed to send HTTP request")
      .map_err(|err| {
        error!(err.to_string());
        AnyhowError::from(err)
      })?;
    let content = response
      .bytes()
      .await
      .context("Failed to read response bytes")
      .map_err(|err| {
        error!(err.to_string());
        AnyhowError::from(err)
      })?;

    let content_str = std::str::from_utf8(&content)
      .context("Failed to convert response bytes to string")
      .map_err(|err| {
        error!(err.to_string());
        AnyhowError::from(err)
      })?;

    let replaced_content = content_str;

    fs::create_dir_all(HTTP_CACHE_DIR)
      .context("Failed to create cache directory")
      .map_err(|err| {
        error!(err.to_string());
        AnyhowError::from(err)
      })?;
    fs::write(&cache_path, &replaced_content)
      .context("Failed to write cache content")
      .map_err(|err| {
        error!(err.to_string());
        AnyhowError::from(err)
      })?;

    return Ok(Some(Content::Buffer(
      replaced_content.to_string().into_bytes(),
    )));
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
