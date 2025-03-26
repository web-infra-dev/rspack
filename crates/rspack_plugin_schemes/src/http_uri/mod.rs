mod http_cache;
mod lockfile;

use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use http_cache::{fetch_content, FetchResultType};
pub use http_cache::{HttpClient, HttpResponse};
use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::{
  get_scheme, ApplyContext, CompilerOptions, Content, ModuleFactoryCreateData,
  NormalModuleFactoryResolveForScheme, NormalModuleFactoryResolveInScheme,
  NormalModuleReadResource, Plugin, PluginContext, ResourceData, Scheme,
};
use rspack_error::Result;
use rspack_fs::WritableFileSystem;
use rspack_hook::{plugin, plugin_hook};
use rspack_util::asset_condition::{AssetCondition, AssetConditions};

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
  async fn respond_with_url_module(
    &self,
    url: url::Url,
    resource_data: &mut ResourceData,
  ) -> Result<bool> {
    let fetch_result = fetch_content(url.as_str(), &self.options)
      .await
      .map_err(rspack_error::AnyhowError::from)?;

    if let FetchResultType::Content(content_result) = fetch_result {
      resource_data.set_resource(url.to_string());

      let path_str = format!("{}{}", url.origin().ascii_serialization(), url.path());
      resource_data.set_path(path_str);

      if let Some(query) = url.query() {
        resource_data.set_query(format!("?{}", query));
      } else {
        resource_data.set_query_optional(None);
      }

      if let Some(fragment) = url.fragment() {
        resource_data.set_fragment(format!("#{}", fragment));
      } else {
        resource_data.set_fragment_optional(None);
      }

      let resolved = content_result.resolved();
      if let Ok(resolved_url) = url::Url::parse(resolved) {
        if let Ok(context_url) = resolved_url.join("./") {
          let context_str = context_url.as_str();
          let _context = if let Some(stripped) = context_str.strip_suffix('/') {
            stripped
          } else {
            context_str
          };
        }
      }

      resource_data.set_mimetype(content_result.content_type().to_string());

      return Ok(true);
    }

    Ok(false)
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
  scheme: &Scheme,
) -> Result<Option<bool>> {
  if scheme.is_http() || scheme.is_https() {
    // Parse the URL
    match url::Url::parse(&resource_data.resource) {
      Ok(url) => match self.respond_with_url_module(url, resource_data).await {
        Ok(true) => Ok(Some(true)),
        Ok(false) => Ok(None),
        Err(e) => Err(e),
      },
      Err(_) => Ok(None),
    }
  } else {
    Ok(None)
  }
}

#[plugin_hook(NormalModuleFactoryResolveInScheme for HttpUriPlugin)]
async fn resolve_in_scheme(
  &self,
  data: &mut ModuleFactoryCreateData,
  resource_data: &mut ResourceData,
  _scheme: &Scheme,
) -> Result<Option<bool>> {
  if !matches!(
    get_scheme(data.context.as_str()),
    Scheme::Http | Scheme::Https
  ) {
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

  // Join the base URL with the resource
  match base_url.join(&resource_url) {
    Ok(url) => match self.respond_with_url_module(url, resource_data).await {
      Ok(true) => return Ok(Some(true)),
      Ok(false) => return Ok(None),
      Err(e) => return Err(e),
    },
    Err(_) => return Ok(None),
  }
}

#[plugin_hook(NormalModuleReadResource for HttpUriPlugin)]
async fn read_resource(&self, resource_data: &ResourceData) -> Result<Option<Content>> {
  if (resource_data.get_scheme().is_http() || resource_data.get_scheme().is_https())
    && EXTERNAL_HTTP_REQUEST.is_match(&resource_data.resource)
  {
    let fetch_result = fetch_content(&resource_data.resource, &self.options)
      .await
      .map_err(rspack_error::AnyhowError::from)?;

    if let FetchResultType::Content(content_result) = fetch_result {
      return Ok(Some(Content::from(content_result.content().to_vec())));
    }
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
