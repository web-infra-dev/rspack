mod http_cache;
mod lockfile;

use std::{fmt::Debug, sync::Arc};

use http_cache::{ContentFetchResult, FetchResultType, fetch_content};
pub use http_cache::{HttpClient, HttpResponse};
use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::{
  Content, ModuleFactoryCreateData, NormalModuleFactoryResolveForScheme,
  NormalModuleFactoryResolveInScheme, NormalModuleReadResource, Plugin, ResourceData, Scheme,
};
use rspack_error::{AnyhowResultToRspackResultExt, Result, error};
use rspack_fs::{ReadableFileSystem, WritableFileSystem};
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

async fn get_info(url: &str, options: &HttpUriPluginOptions) -> Result<ContentFetchResult> {
  // Check if the URL is allowed
  if !options.allowed_uris.is_allowed(url) {
    return Err(error!(
      "{} doesn't match the allowedUris policy. These URIs are allowed:\n{}",
      url,
      options.allowed_uris.get_allowed_uris_description(),
    ));
  }
  resolve_content(url, options, 0).await
}

const MAX_REDIRECTS: usize = 5;

/// Sanitize URL for inclusion in error messages
#[allow(clippy::disallowed_methods)]
fn sanitize_url_for_error(href: &str) -> String {
  match Url::parse(href) {
    Ok(u) => format!("{}//{}", u.scheme(), u.host_str().unwrap_or("")),
    Err(_) => href
      .chars()
      .take(200)
      .collect::<String>()
      .replace(['\r', '\n'].as_ref(), ""),
  }
}

/// Validate redirect location against allowed URIs and protocol constraints
fn validate_redirect_location(
  location: &str,
  base: &str,
  options: &HttpUriPluginOptions,
) -> Result<String> {
  // Parse the redirect location relative to the base URL
  let base_url =
    Url::parse(base).map_err(|_| error!("Invalid base URL: {}", sanitize_url_for_error(base)))?;

  let next_url = base_url
    .join(location)
    .map_err(|_| error!("Invalid redirect URL: {}", sanitize_url_for_error(location)))?;

  // Ensure redirect uses only http or https protocol
  if next_url.scheme() != "http" && next_url.scheme() != "https" {
    return Err(error!(
      "Redirected URL uses disallowed protocol: {}",
      sanitize_url_for_error(next_url.as_str())
    ));
  }

  // Ensure redirect target is still within allowed URIs
  if !options.allowed_uris.is_allowed(next_url.as_str()) {
    return Err(error!(
      "{} doesn't match the allowedUris policy after redirect. These URIs are allowed:\n{}",
      next_url.as_str(),
      options.allowed_uris.get_allowed_uris_description(),
    ));
  }

  Ok(next_url.to_string())
}

// recursively handle http redirect
async fn resolve_content(
  url: &str,
  options: &HttpUriPluginOptions,
  redirect_count: usize,
) -> Result<ContentFetchResult> {
  let result = fetch_content(url, options)
    .await
    .to_rspack_result_from_anyhow()?;
  match result {
    FetchResultType::Content(content) => Ok(content),
    FetchResultType::Redirect(redirect) => {
      // Validate redirect before following
      let validated_location = validate_redirect_location(&redirect.location, url, options)?;

      // Check redirect limit
      if redirect_count >= MAX_REDIRECTS {
        return Err(error!("Too many redirects"));
      }

      Box::pin(resolve_content(
        &validated_location,
        options,
        redirect_count + 1,
      ))
      .await
    }
  }
}

/// Scheme only for http and https
fn parse_url_as_http(url: &str) -> Option<Url> {
  let url = Url::parse(url).ok()?;
  if url.scheme() != "http" && url.scheme() != "https" {
    return None;
  }
  Some(url)
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
    let mut path = url.origin().ascii_serialization();
    path.push_str(url.path());
    resource_data.set_path(path);
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
  let Some(url) = parse_url_as_http(resource_data.resource()) else {
    return Ok(None);
  };

  match self
    .respond_with_url_module(resource_data, &url, None)
    .await
  {
    Ok(true) => Ok(Some(true)),
    Ok(false) => Ok(None),
    Err(e) => Err(e),
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
    .is_none_or(|dep| dep.dependency_type().as_str() != "url");

  // Only handle relative urls (./xxx, ../xxx, /xxx, //xxx) and non-url dependencies
  let resource = resource_data.resource();
  if is_not_url_dependency
    && (!resource.starts_with("./")
      && !resource.starts_with("../")
      && !resource.starts_with("/")
      && !resource.starts_with("//"))
  {
    return Ok(None);
  }

  // Parse the base URL from context
  let Some(base_url) = parse_url_as_http(&format!("{}/", data.context)) else {
    return Ok(None);
  };

  // Join the base URL with the resource
  match base_url.join(resource_data.resource()) {
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
async fn read_resource(
  &self,
  resource_data: &ResourceData,
  _fs: &Arc<dyn ReadableFileSystem>,
) -> Result<Option<Content>> {
  if (resource_data.get_scheme().is_http() || resource_data.get_scheme().is_https())
    && EXTERNAL_HTTP_REQUEST.is_match(resource_data.resource())
  {
    let content_result = get_info(resource_data.resource(), &self.options).await?;

    return Ok(Some(Content::from(content_result.content().to_vec())));
  }
  Ok(None)
}

impl Plugin for HttpUriPlugin {
  fn name(&self) -> &'static str {
    "rspack.HttpUriPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .normal_module_factory_hooks
      .resolve_for_scheme
      .tap(resolve_for_scheme::new(self));
    ctx
      .normal_module_factory_hooks
      .resolve_in_scheme
      .tap(resolve_in_scheme::new(self));
    ctx
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
      AssetCondition::String(s) => s.clone(),
      AssetCondition::Regexp(r) => r.to_source_string(),
    }
  }
}

// align with https://github.com/webpack/webpack/blob/dec18718be5dfba28f067fb3827dd620a1f33667/lib/schemes/HttpUriPlugin.js#L1154
// set
fn get_resource_context(result_entry_resolved: &str) -> Option<String> {
  // Parse the resolved URL
  let base_url = parse_url_as_http(result_entry_resolved)?;

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
