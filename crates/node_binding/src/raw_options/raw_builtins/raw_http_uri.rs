use std::{
  collections::HashMap,
  fmt::Debug,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
};

use async_trait::async_trait;
use napi::bindgen_prelude::{Buffer, Either, Promise};
use napi_derive::napi;
use once_cell::sync::OnceCell;
use rspack_error::AnyhowError;
use rspack_fs::WritableFileSystem;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_plugin_schemes::{
  HttpClient, HttpResponse, HttpUriOptionsAllowedUris, HttpUriPlugin, HttpUriPluginOptions,
};
use rspack_regex::RspackRegex;
use rspack_util::asset_condition::{AssetCondition, AssetConditions};

#[napi(object)]
#[derive(Debug)]
pub struct RawHttpUriPluginOptions {
  #[napi(ts_type = "(string | RegExp)[]")]
  pub allowed_uris: Option<Vec<Either<String, RspackRegex>>>,
  pub cache_location: Option<String>,
  pub frozen: Option<bool>,
  pub lockfile_location: Option<String>,
  pub proxy: Option<String>,
  pub upgrade: Option<bool>,
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

static JS_HTTP_CLIENT: OnceCell<JsHttpClient> = OnceCell::new();
static HTTP_CLIENT_REGISTERED: AtomicBool = AtomicBool::new(false);

#[napi(
  ts_type = "(http_client: (url: string, headers: Record<string, string>) => Promise<{ status: number, headers: Record<string, string>, body: Buffer }>):void"
)]
pub fn register_http_client(
  http_client: ThreadsafeFunction<(String, HashMap<String, String>), Promise<JsHttpResponseRaw>>,
) {
  let client = JsHttpClient {
    function: http_client,
  };

  let _ = JS_HTTP_CLIENT.set(client);
  HTTP_CLIENT_REGISTERED.store(true, Ordering::SeqCst);
}

pub fn create_http_uri_plugin(
  allowed_uris: Option<Vec<Either<String, RspackRegex>>>,
  cache_location: Option<String>,
  frozen: Option<bool>,
  lockfile_location: Option<String>,
  proxy: Option<String>,
  upgrade: Option<bool>,
  filesystem: Arc<dyn WritableFileSystem>,
) -> Result<HttpUriPlugin, AnyhowError> {
  let allowed_uris = match allowed_uris {
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

  let http_client = if HTTP_CLIENT_REGISTERED.load(Ordering::SeqCst) {
    let js_client = JS_HTTP_CLIENT
      .get()
      .expect("HTTP client not available from JavaScript side")
      .clone();

    Some(Arc::new(js_client) as Arc<dyn HttpClient>)
  } else {
    return Err(AnyhowError::from(anyhow::anyhow!(
      "HTTP client not registered from JavaScript side"
    )));
  };

  let options = HttpUriPluginOptions {
    allowed_uris,
    cache_location,
    frozen,
    lockfile_location,
    proxy,
    upgrade,
    filesystem,
    http_client,
  };

  Ok(HttpUriPlugin::new(options))
}

pub fn get_http_uri_plugin(options: RawHttpUriPluginOptions) -> Box<dyn rspack_core::Plugin> {
  // Use NativeFileSystem for HTTP caching
  let fs = Arc::new(rspack_fs::NativeFileSystem::new(false));

  // Create the plugin with the provided options
  let plugin = create_http_uri_plugin(
    options.allowed_uris,
    options.cache_location,
    options.frozen,
    options.lockfile_location,
    options.proxy,
    options.upgrade,
    fs,
  )
  .expect("Failed to create HttpUriPlugin");

  Box::new(plugin)
}
