mod http_cache;
mod lockfile;

use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use http_cache::{fetch_content, ContentFetchResult, FetchResultType};
pub use http_cache::{HttpClient, HttpResponse};
use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::{
  ApplyContext, CompilerOptions, Content, ModuleFactoryCreateData,
  NormalModuleFactoryResolveForScheme, NormalModuleFactoryResolveInScheme,
  NormalModuleReadResource, Plugin, PluginContext, ResourceData, Scheme,
};
use rspack_error::{
  miette::{self, diagnostic},
  AnyhowResultToRspackResultExt, Result,
};
use rspack_fs::WritableFileSystem;
use rspack_hook::{plugin, plugin_hook};
use rspack_util::asset_condition::{AssetCondition, AssetConditions};
use url::Url;

static EXTERNAL_HTTP_REQUEST: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^(//|https?://|#)").expect("Invalid regex"));

#[plugin]
#[derive(Debug)]
pub struct HttpUriPlugin {
  options: HttpUriPluginOptions,
}

async fn get_info(url: &str, options: &HttpUriPluginOptions) -> miette::Result<ContentFetchResult> {
  // Check if the URL is allowed
  if !options.allowed_uris.is_allowed(url) {
    return Err(
      diagnostic!(
        "{} doesn't match the allowedUris policy. These URIs are allowed:\n{}",
        url,
        options.allowed_uris.get_allowed_uris_description(),
      )
      .into(),
    );
  }
  resolve_content(url, options).await
}

// recursively handle http redirect
async fn resolve_content(
  url: &str,
  options: &HttpUriPluginOptions,
) -> miette::Result<ContentFetchResult> {
  let result = fetch_content(url, options)
    .await
    .to_rspack_result_from_anyhow()?;
  match result {
    FetchResultType::Content(content) => Ok(content),
    FetchResultType::Redirect(redirect) => {
      Box::pin(resolve_content(&redirect.location, options)).await
    }
  }
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
    let resolved_result = get_info(url.as_str(), &self.options).await?;

    let context = get_resource_context(&resolved_result.entry.resolved);
    resource_data.set_context(context);
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
  // Try to parse the URL and handle it
  match Url::parse(&resource_data.resource) {
    Ok(url) => match self
      .respond_with_url_module(resource_data, &url, None)
      .await
    {
      Ok(true) => Ok(Some(true)),
      Ok(false) => Ok(None),
      Err(e) => Err(e),
    },
    Err(_) => Ok(None),
  }
}

#[plugin_hook(NormalModuleFactoryResolveInScheme for HttpUriPlugin)]
async fn resolve_in_scheme(
  &self,
  data: &mut ModuleFactoryCreateData,
  resource_data: &mut ResourceData,
  _scheme: &Scheme,
) -> Result<Option<bool>> {
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
    return Ok(None);
  }

  // Parse the base URL from context
  let base_url = match Url::parse(&format!("{}/", data.context)) {
    Ok(url) => url,
    Err(_) => return Ok(None),
  };

  // Join the base URL with the resource
  match base_url.join(&resource_data.resource) {
    Ok(url) => match self
      .respond_with_url_module(resource_data, &url, None)
      .await
    {
      Ok(true) => Ok(Some(true)),
      Ok(false) => Ok(None),
      Err(e) => Err(e),
    },
    Err(_) => Ok(None),
  }
}

#[plugin_hook(NormalModuleReadResource for HttpUriPlugin)]
async fn read_resource(&self, resource_data: &ResourceData) -> Result<Option<Content>> {
  if (resource_data.get_scheme().is_http() || resource_data.get_scheme().is_https())
    && EXTERNAL_HTTP_REQUEST.is_match(&resource_data.resource)
  {
    let content_result = get_info(&resource_data.resource, &self.options).await?;

    return Ok(Some(Content::from(content_result.content().to_vec())));
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

// align with https://github.com/webpack/webpack/blob/dec18718be5dfba28f067fb3827dd620a1f33667/lib/schemes/HttpUriPlugin.js#L1154
// set
fn get_resource_context(result_entry_resolved: &str) -> Option<String> {
  // Parse the resolved URL
  if let Ok(base_url) = Url::parse(result_entry_resolved) {
    // Resolve the relative path "." against the base URL
    if let Ok(resolved_url) = base_url.join(".") {
      // Convert the resolved URL to a string
      let mut href = resolved_url.to_string();

      // Remove the trailing slash if it exists
      if href.ends_with('/') {
        href.pop();
      }

      // Return the context as a string
      return Some(href);
    }
  }

  // Return None if parsing or joining fails
  None
}
#[cfg(test)]
mod test {
  use crate::http_uri::get_resource_context;

  #[test]
  fn test_get_resource_context() {
    assert_eq!(
      get_resource_context("https://www.unpkg.com/react-dom@18.3.1/index.js"),
      Some("https://www.unpkg.com/react-dom@18.3.1".to_string())
    );
    assert_eq!(
      get_resource_context("https://www.unpkg.com/react-dom@18.3.1/"),
      Some("https://www.unpkg.com/react-dom@18.3.1".to_string())
    );
    // FIXME: add more test cases
  }
}
