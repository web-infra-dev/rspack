use std::fs;
use std::path::Path;

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
use rspack_error::{AnyhowError, Result}; // Removed `Error` import
use rspack_hook::{plugin, plugin_hook}; // Add this import

static EXTERNAL_HTTP_REQUEST: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^(//|https?://|#)").expect("Invalid regex"));

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

    let content_str = std::str::from_utf8(&content)
      .context("Failed to convert response bytes to string")
      .map_err(|err| {
        error!(err.to_string());
        AnyhowError::from(err)
      })?;

    let base_url = &resource_data.resource;
    let origin = match url::Url::parse(base_url) {
      Ok(url) => url.origin().ascii_serialization(),
      Err(_) => "".to_string(),
    };
    let replaced_content = content_str
      .replace("import \"/", &format!("import \"{}/", origin))
      .replace("from \"/", &format!("from \"{}/", origin));
    dbg!(&replaced_content);
    let final_content = replaced_content.into_bytes();

    fs::create_dir_all(HTTP_CACHE_DIR)
      .context("Failed to create cache directory")
      .map_err(|err| {
        error!(err.to_string());
        AnyhowError::from(err)
      })?;
    fs::write(&cache_path, &content)
      .context("Failed to write cache content")
      .map_err(|err| {
        error!(err.to_string());
        AnyhowError::from(err)
      })?;

    return Ok(Some(Content::Buffer(final_content.to_vec())));
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
