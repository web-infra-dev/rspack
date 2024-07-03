use std::sync::Arc;

use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::{
  get_scheme, ApplyContext, CompilerOptions, Content, ModuleFactoryCreateData,
  NormalModuleFactoryResolveForScheme, NormalModuleFactoryResolveInScheme,
  NormalModuleReadResource, Plugin, PluginContext, ResourceData, Scheme,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

use crate::http_cache::{fetch_content, FetchResultType};
use crate::lockfile::LockfileEntry;

static EXTERNAL_HTTP_REQUEST: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^(//|https?://|#)").expect("Invalid regex"));

static LOCKFILE_LOCATION: &str = "default_lockfile_location";
static CACHE_LOCATION: &str = "default_cache_location";
static UPGRADE: bool = true;
static FROZEN: bool = false;
static ALLOWED_URIS: &[&str] = &["http://example.com"];
static PROXY: &str = "http://proxy.example.com";

#[plugin]
#[derive(Debug, Default)]
pub struct HttpUriPlugin {
  options: HttpUriPluginOptions,
  lockfile_cache: LockfileCache,
}

#[derive(Debug, Default)]
pub struct HttpUriPluginOptions {}

impl HttpUriPlugin {
  pub fn new(options: HttpUriPluginOptions) -> Self {
    Self::new_inner(options, LockfileCache::default())
  }
}

#[derive(Debug, Default)]
pub struct LockfileCache {
  lockfile: Lockfile,
  snapshot: String, // Placeholder for the actual snapshot type
}

impl LockfileCache {
  pub fn new() -> Self {
    Self {
      lockfile: Lockfile::default(), // Use the correct initializer
      snapshot: String::new(),       // Initialize with default values
    }
  }
}

#[derive(Debug)]
pub struct Lockfile {
  pub version: u32,
  pub entries: Vec<LockfileEntry>,
}

impl Default for Lockfile {
  fn default() -> Self {
    Lockfile {
      version: 1,          // or the appropriate default value
      entries: Vec::new(), // or the appropriate default value
    }
  }
}

#[plugin_hook(NormalModuleFactoryResolveForScheme for HttpUriPlugin)]
async fn resolve_for_scheme(
  &self,
  _data: &mut ModuleFactoryCreateData,
  resource_data: &mut ResourceData,
) -> Result<Option<bool>> {
  if resource_data.get_scheme().is_http() && EXTERNAL_HTTP_REQUEST.is_match(&resource_data.resource)
  {
    return Ok(None);
  }
  Ok(None)
}

#[plugin_hook(NormalModuleFactoryResolveInScheme for HttpUriPlugin)]
async fn resolve_in_scheme(
  &self,
  data: &mut ModuleFactoryCreateData,
  resource_data: &mut ResourceData,
) -> Result<Option<bool>> {
  if !matches!(get_scheme(data.context.as_str()), Scheme::Http) {
    return Ok(None);
  }

  let base_url = match url::Url::parse(data.context.as_str()) {
    Ok(url) => url,
    Err(_) => return Ok(None),
  };

  let resource_url = match url::Url::parse(&resource_data.resource) {
    Ok(url) if url.scheme() == "http" || url.scheme() == "https" => return Ok(None),
    Ok(_) | Err(_) => resource_data.resource.clone(),
  };

  resource_data.set_resource(
    base_url
      .join(&resource_url)
      .map(|url| url.to_string())
      .unwrap_or_else(|_| resource_data.resource.clone()),
  );

  Ok(None)
}

#[plugin_hook(NormalModuleReadResource for HttpUriPlugin)]
async fn read_resource(&self, resource_data: &ResourceData) -> Result<Option<Content>> {
  if resource_data.get_scheme().is_http() && EXTERNAL_HTTP_REQUEST.is_match(&resource_data.resource)
  {
    let fetch_result = fetch_content(&resource_data.resource)
      .await
      .map_err(rspack_error::AnyhowError::from)?;
    match fetch_result {
      FetchResultType::Content(content_result) => {
        let content = Content::from(content_result.content().clone());
        return Ok(Some(content));
      }
      FetchResultType::Redirect(_) => return Ok(None),
    }
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
      .normal_module_factory_hooks
      .resolve_in_scheme
      .tap(resolve_in_scheme::new(self));
    ctx
      .context
      .normal_module_hooks
      .read_resource
      .tap(read_resource::new(self));
    Ok(())
  }
}
