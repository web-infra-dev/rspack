use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use napi::bindgen_prelude::{Buffer, Either, FnArgs, Promise};
use napi_derive::napi;
use rspack_fs::WritableFileSystem;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_plugin_schemes::{
  HttpClient, HttpResponse, HttpUriOptionsAllowedUris, HttpUriPlugin, HttpUriPluginOptions,
};
use rspack_regex::RspackRegex;
use rspack_util::asset_condition::{AssetCondition, AssetConditions};
use rustc_hash::FxHashMap as HashMap;

type HttpClientRequest =
  ThreadsafeFunction<FnArgs<(String, HashMap<String, String>)>, Promise<JsHttpResponseRaw>>;

#[napi(object, object_to_js = false)]
#[derive(Debug)]
pub struct RawHttpUriPluginOptions {
  #[napi(ts_type = "(string | RegExp)[]")]
  pub allowed_uris: Vec<Either<String, RspackRegex>>,
  pub lockfile_location: Option<String>,
  pub cache_location: Option<String>,
  pub upgrade: bool,
  // pub proxy: Option<String>,
  // pub frozen: Option<bool>,
  #[napi(ts_type = "(url: string, headers: Record<string, string>) => Promise<JsHttpResponseRaw>")]
  pub http_client: HttpClientRequest,
}

#[napi(object)]
pub struct JsHttpResponseRaw {
  pub status: u16,
  pub headers: HashMap<String, String>,
  pub body: Buffer,
}

type JsHttpClientFunction =
  ThreadsafeFunction<FnArgs<(String, HashMap<String, String>)>, Promise<JsHttpResponseRaw>>;

#[derive(Debug, Clone)]
pub struct JsHttpClient {
  function: JsHttpClientFunction,
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
      .call_with_promise((url_owned, headers_owned).into())
      .await
      .map_err(|e| anyhow::anyhow!("Error calling JavaScript HTTP client: {}", e))?;

    Ok(HttpResponse {
      status: result.status,
      headers: result.headers,
      body: result.body,
    })
  }
}

fn create_http_uri_plugin_options(
  options: RawHttpUriPluginOptions,
  filesystem: Arc<dyn WritableFileSystem>,
) -> HttpUriPluginOptions {
  let allowed_uris = HttpUriOptionsAllowedUris::new(AssetConditions::Multiple(
    options
      .allowed_uris
      .into_iter()
      .map(|condition| match condition {
        Either::A(string) => AssetCondition::String(string),
        Either::B(regex) => AssetCondition::Regexp(regex),
      })
      .collect(),
  ));

  let http_client = Arc::new(JsHttpClient {
    function: options.http_client,
  });

  HttpUriPluginOptions {
    allowed_uris,
    lockfile_location: options.lockfile_location,
    cache_location: options.cache_location,
    upgrade: options.upgrade,
    // proxy: options.proxy,
    // frozen: options.frozen,
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
