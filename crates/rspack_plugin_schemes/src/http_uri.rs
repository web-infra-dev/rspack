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
use crate::lockfile::LockfileCache;

static EXTERNAL_HTTP_REQUEST: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^(//|https?://|#)").expect("Invalid regex"));

#[plugin]
#[derive(Debug, Default)]
pub struct HttpUriPlugin {
  options: HttpUriPluginOptions,
  #[allow(dead_code)]
  lockfile_cache: LockfileCache,
}

impl HttpUriPlugin {
  pub fn new(options: HttpUriPluginOptions) -> Self {
    Self::new_inner(options, LockfileCache::default())
  }
}

#[derive(Debug, Default)]
pub struct HttpUriPluginOptions {
  pub allowed_uris: HttpUriOptionsAllowedUris,
  pub cache_location: Option<String>,
  pub frozen: Option<bool>,
  pub lockfile_location: Option<String>,
  pub proxy: Option<String>,
  pub upgrade: Option<bool>,
}

#[plugin_hook(NormalModuleFactoryResolveForScheme for HttpUriPlugin)]
async fn resolve_for_scheme(
  &self,
  _data: &mut ModuleFactoryCreateData,
  resource_data: &mut ResourceData,
  scheme: &Scheme,
) -> Result<Option<bool>> {
  if scheme.is_http() && EXTERNAL_HTTP_REQUEST.is_match(&resource_data.resource) {
    return Ok(None);
  }
  Ok(None)
}

#[plugin_hook(NormalModuleFactoryResolveInScheme for HttpUriPlugin)]
async fn resolve_in_scheme(
  &self,
  data: &mut ModuleFactoryCreateData,
  resource_data: &mut ResourceData,
  _scheme: &Scheme,
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
    let fetch_result = fetch_content(&resource_data.resource, &self.options)
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

#[derive(Debug, Default)]
pub struct HttpUriOptionsAllowedUris;
