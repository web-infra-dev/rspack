use std::{collections::HashMap, fmt::Debug, sync::Arc};

use async_trait::async_trait;
use napi::bindgen_prelude::{Buffer, Either, Promise};
use napi_derive::napi;
use rspack_fs::WritableFileSystem;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_plugin_schemes::{
  HttpClient, HttpResponse, HttpUriOptionsAllowedUris, HttpUriPlugin, HttpUriPluginOptions,
};
use rspack_regex::RspackRegex;
use rspack_util::asset_condition::{AssetCondition, AssetConditions};

#[napi(object, object_to_js = false)]
#[derive(Debug)]
pub struct RawHttpUriPluginOptions {
  #[napi(ts_type = "(string | RegExp)[]")]
  pub allowed_uris: Option<Vec<Either<String, RspackRegex>>>,
  pub cache_location: Option<String>,
  pub frozen: Option<bool>,
  pub lockfile_location: Option<String>,
  pub proxy: Option<String>,
  pub upgrade: Option<bool>,
  #[napi(ts_type = "(url: string, headers: Record<string, string>) => Promise<JsHttpResponseRaw>")]
  pub http_client:
    ThreadsafeFunction<(String, HashMap<String, String>), Promise<JsHttpResponseRaw>>,
}

#[napi(object)]
pub struct JsHttpResponseRaw {
  pub status: u16,
  pub headers: HashMap<String, String>,
  pub body: Buffer,
}

#[derive(Debug, Clone)]
pub struct JsHttpClient {
  function: ThreadsafeFunction<(String, HashMap<String, String>), Promise<JsHttpResponseRaw>>,
}

#[async_trait]
impl HttpClient for JsHttpClient {
  async fn get(
    &self,
    url: &str,
    headers: &HashMap<String, String>,
  ) -> anyhow::Result<HttpResponse> {
    let url_owned = url.to_string();
    let headers_owned = headers.clone();
    let func = self.function.clone();

    let result = func
      .call_with_promise((url_owned, headers_owned))
      .await
      .map_err(|e| anyhow::anyhow!("Error calling JavaScript HTTP client: {}", e))?;

    Ok(HttpResponse {
      status: result.status,
      headers: result.headers,
      body: result.body.to_vec(),
    })
  }
}

fn create_http_uri_plugin_options(
  options: RawHttpUriPluginOptions,
  filesystem: Arc<dyn WritableFileSystem>,
) -> HttpUriPluginOptions {
  let allowed_uris = match options.allowed_uris {
    Some(conditions) => {
      let asset_conditions = conditions
        .into_iter()
        .map(|condition| match condition {
          Either::A(string) => AssetCondition::String(string),
          Either::B(regex) => AssetCondition::Regexp(regex),
        })
        .collect();
      HttpUriOptionsAllowedUris::from_asset_conditions(AssetConditions::Multiple(asset_conditions))
    }
    None => HttpUriOptionsAllowedUris::default(),
  };

  let http_client = Arc::new(JsHttpClient {
    function: options.http_client,
  });

  HttpUriPluginOptions {
    allowed_uris,
    cache_location: options.cache_location,
    frozen: options.frozen,
    lockfile_location: options.lockfile_location,
    proxy: options.proxy,
    upgrade: options.upgrade,
    http_client,
    filesystem,
  }
}

pub fn get_http_uri_plugin(options: RawHttpUriPluginOptions) -> Box<dyn rspack_core::Plugin> {
  // Use NativeFileSystem for HTTP caching
  let fs = Arc::new(rspack_fs::NativeFileSystem::new(false));

  // Create the plugin with the provided options
  let options = create_http_uri_plugin_options(options, fs);
  let plugin = HttpUriPlugin::new(options);

  Box::new(plugin)
}
