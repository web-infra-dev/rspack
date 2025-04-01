mod http_cache;
mod lockfile;

use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use http_cache::{fetch_content, FetchResultType};
pub use http_cache::{HttpClient, HttpResponse};
use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::{
  ApplyContext, CompilerOptions, Content, ModuleFactoryCreateData,
  NormalModuleFactoryResolveForScheme, NormalModuleFactoryResolveInScheme,
  NormalModuleReadResource, Plugin, PluginContext, ResourceData, Scheme,
};
use rspack_error::Result;
use rspack_fs::WritableFileSystem;
use rspack_hook::{plugin, plugin_hook};
use rspack_util::asset_condition::{AssetCondition, AssetConditions};
use tracing::{debug, info, warn};
use url::Url;

static EXTERNAL_HTTP_REQUEST: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^(//|https?://|#)").expect("Invalid regex"));

#[plugin]
#[derive(Debug)]
pub struct HttpUriPlugin {
  options: HttpUriPluginOptions,
}

impl HttpUriPlugin {
  pub fn new(options: HttpUriPluginOptions) -> Self {
    Self::new_inner(options)
  }
  pub async fn respond_with_url_module(
    &self,
    resource_data: &mut ResourceData,
    url: &Url,
    mimetype: Option<String>,
  ) -> Result<bool> {
    resource_data.set_resource(url.to_string());
    resource_data.set_path(url.origin().ascii_serialization() + url.path());
    if let Some(query) = url.query() {
      resource_data.set_query(query.to_string());
    }
    if let Some(fragment) = url.fragment() {
      resource_data.set_fragment(fragment.to_string());
    }
    if let Some(mime) = mimetype {
      resource_data.set_mimetype(mime);
    }
    Ok(true)
  }
}

#[derive(Debug)]
pub struct HttpUriPluginOptions {
  pub allowed_uris: HttpUriOptionsAllowedUris,
  pub lockfile_location: Option<String>,
  pub cache_location: Option<String>,
  pub upgrade: bool,
  // pub proxy: Option<String>,
  // pub frozen: Option<bool>,
  pub filesystem: Arc<dyn WritableFileSystem>,
  pub http_client: Arc<dyn HttpClient>,
}

#[plugin_hook(NormalModuleFactoryResolveForScheme for HttpUriPlugin)]
async fn resolve_for_scheme(
  &self,
  _data: &mut ModuleFactoryCreateData,
  resource_data: &mut ResourceData,
  _scheme: &Scheme,
) -> Result<Option<bool>> {
  // Adding debug logging for resource path
  debug!(
    "[HttpUriPlugin::resolve_for_scheme] Processing resource: {}",
    resource_data.resource
  );

  // Try to parse the URL and handle it
  match Url::parse(&resource_data.resource) {
    Ok(url) => {
      debug!(
        "[HttpUriPlugin::resolve_for_scheme] Successfully parsed URL: {}",
        url
      );
      match self
        .respond_with_url_module(resource_data, &url, None)
        .await
      {
        Ok(true) => {
          debug!("[HttpUriPlugin::resolve_for_scheme] Successfully responded with URL module");
          Ok(Some(true))
        }
        Ok(false) => {
          debug!("[HttpUriPlugin::resolve_for_scheme] Not responding with URL module");
          Ok(None)
        }
        Err(e) => {
          warn!(
            "[HttpUriPlugin::resolve_for_scheme] Error responding with URL module: {:?}",
            e
          );
          Err(e)
        }
      }
    }
    Err(e) => {
      debug!(
        "[HttpUriPlugin::resolve_for_scheme] Failed to parse URL: {:?}",
        e
      );
      Ok(None)
    }
  }
}

#[plugin_hook(NormalModuleFactoryResolveInScheme for HttpUriPlugin)]
async fn resolve_in_scheme(
  &self,
  data: &mut ModuleFactoryCreateData,
  resource_data: &mut ResourceData,
  _scheme: &Scheme,
) -> Result<Option<bool>> {
  // Adding debug logging for resource path
  debug!(
    "[HttpUriPlugin::resolve_in_scheme] Processing resource: {}, context: {}",
    resource_data.resource, data.context
  );

  // Check if the dependency type is "url", similar to webpack's check
  let is_not_url_dependency = data
    .dependencies
    .first()
    .and_then(|dep| dep.as_module_dependency())
    .map(|dep| dep.dependency_type().as_str() != "url")
    .unwrap_or(true);

  // Only handle relative urls (./xxx, ../xxx, /xxx, //xxx) and non-url dependencies
  if is_not_url_dependency
    && (!resource_data.resource.starts_with("./")
      && !resource_data.resource.starts_with("../")
      && !resource_data.resource.starts_with("/")
      && !resource_data.resource.starts_with("//"))
  {
    debug!("[HttpUriPlugin::resolve_in_scheme] Not a relative URL or URL dependency, skipping");
    return Ok(None);
  }

  // Parse the base URL from context
  let base_url = match Url::parse(&format!("{}/", data.context)) {
    Ok(url) => {
      debug!("[HttpUriPlugin::resolve_in_scheme] Base URL: {}", url);
      url
    }
    Err(e) => {
      debug!(
        "[HttpUriPlugin::resolve_in_scheme] Failed to parse base URL: {:?}",
        e
      );
      return Ok(None);
    }
  };

  // Join the base URL with the resource
  match base_url.join(&resource_data.resource) {
    Ok(url) => {
      debug!("[HttpUriPlugin::resolve_in_scheme] Joined URL: {}", url);
      if url.to_string().contains("react-refresh") || url.to_string().contains("reactRefresh") {
        info!(
          "[HttpUriPlugin::resolve_in_scheme] Detected React Refresh URL: {}",
          url
        );
      }
      match self
        .respond_with_url_module(resource_data, &url, None)
        .await
      {
        Ok(true) => {
          debug!("[HttpUriPlugin::resolve_in_scheme] Successfully responded with URL module");
          Ok(Some(true))
        }
        Ok(false) => {
          debug!("[HttpUriPlugin::resolve_in_scheme] Not responding with URL module");
          Ok(None)
        }
        Err(e) => {
          warn!(
            "[HttpUriPlugin::resolve_in_scheme] Error responding with URL module: {:?}",
            e
          );
          Err(e)
        }
      }
    }
    Err(e) => {
      debug!(
        "[HttpUriPlugin::resolve_in_scheme] Failed to join URL: {:?}",
        e
      );
      Ok(None)
    }
  }
}

#[plugin_hook(NormalModuleReadResource for HttpUriPlugin)]
async fn read_resource(&self, resource_data: &ResourceData) -> Result<Option<Content>> {
  debug!(
    "[HttpUriPlugin::read_resource] Processing resource: {}, scheme: {:?}",
    resource_data.resource,
    resource_data.get_scheme()
  );

  if (resource_data.get_scheme().is_http() || resource_data.get_scheme().is_https())
    && EXTERNAL_HTTP_REQUEST.is_match(&resource_data.resource)
  {
    // Add specific logging for React Refresh resources
    if resource_data.resource.contains("react-refresh")
      || resource_data.resource.contains("reactRefresh")
    {
      info!(
        "[HttpUriPlugin::read_resource] Attempting to load React Refresh resource: {}",
        resource_data.resource
      );
    }

    let fetch_result = fetch_content(&resource_data.resource, &self.options)
      .await
      .map_err(|e| {
        warn!(
          "[HttpUriPlugin::read_resource] Error fetching content for {}: {:?}",
          resource_data.resource, e
        );
        rspack_error::AnyhowError::from(e)
      })?;

    if let FetchResultType::Content(content_result) = fetch_result {
      debug!(
        "[HttpUriPlugin::read_resource] Successfully fetched content for {}",
        resource_data.resource
      );
      return Ok(Some(Content::from(content_result.content().to_vec())));
    } else {
      debug!(
        "[HttpUriPlugin::read_resource] No content fetched for {}",
        resource_data.resource
      );
    }
  } else {
    debug!(
      "[HttpUriPlugin::read_resource] Not an HTTP(S) resource or doesn't match external HTTP pattern: {}",
      resource_data.resource
    );
  }
  Ok(None)
}

#[async_trait]
impl Plugin for HttpUriPlugin {
  fn name(&self) -> &'static str {
    "rspack.HttpUriPlugin"
  }

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
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

#[derive(Debug)]
pub struct HttpUriOptionsAllowedUris {
  conditions: AssetConditions,
}

impl HttpUriOptionsAllowedUris {
  pub fn new(conditions: AssetConditions) -> Self {
    Self { conditions }
  }

  pub fn is_allowed(&self, uri: &str) -> bool {
    self.conditions.try_match(uri)
  }

  pub fn get_allowed_uris_description(&self) -> String {
    match &self.conditions {
      AssetConditions::Single(condition) => self.condition_to_string(condition),
      AssetConditions::Multiple(conditions) => conditions
        .iter()
        .map(|c| format!(" - {}", self.condition_to_string(c)))
        .collect::<Vec<_>>()
        .join("\n"),
    }
  }

  fn condition_to_string(&self, condition: &AssetCondition) -> String {
    match condition {
      AssetCondition::String(s) => s.to_string(),
      AssetCondition::Regexp(r) => r.to_source_string(),
    }
  }
}
