use std::{
  collections::HashMap,
  fmt::Debug,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
};

use async_trait::async_trait;
use napi::bindgen_prelude::{Buffer, Promise};
use napi_derive::napi;
use once_cell::sync::OnceCell;
use rspack_error::AnyhowError;
use rspack_fs::WritableFileSystem;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_plugin_schemes::{
  HttpClient, HttpResponse, HttpUriOptionsAllowedUris, HttpUriPlugin, HttpUriPluginOptions,
};

// Type definition for the ThreadsafeFunction used in HTTP requests
type HttpRequestParams = (
  Option<String>,
  Option<String>,
  String,
  HashMap<String, String>,
);

type HttpClientFunction = ThreadsafeFunction<HttpRequestParams, Promise<JsHttpResponseRaw>>;

#[napi(object)]
pub struct JsHttpResponseRaw {
  pub status: u16,
  pub headers: HashMap<String, String>,
  pub body: Buffer,
}

#[derive(Debug, Clone)]
pub struct JsHttpClient {
  function: HttpClientFunction,
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

    println!(
      "[JsHttpClient] Preparing to call JS with URL: '{}'",
      url_owned
    );
    println!("[JsHttpClient] Headers: {:?}", headers_owned);

    let func = self.function.clone();

    let method_str: Option<String> = Some("GET".to_string());
    let url_str: String = url_owned.clone();
    let null_str: Option<String> = None;

    let result = func
      .call_with_promise((null_str, method_str, url_str, headers_owned))
      .await
      .map_err(|e| anyhow::anyhow!("Error calling JavaScript HTTP client: {}", e))?;

    println!(
      "[JsHttpClient] Received response with status: {}",
      result.status
    );

    Ok(HttpResponse {
      status: result.status,
      headers: result.headers,
      body: result.body.to_vec(),
    })
  }
}

static JS_HTTP_CLIENT: OnceCell<JsHttpClient> = OnceCell::new();
static HTTP_CLIENT_REGISTERED: AtomicBool = AtomicBool::new(false);

#[napi]
pub fn register_http_client(http_client: HttpClientFunction) {
  let client = JsHttpClient {
    function: http_client,
  };

  // This is safe because OnceCell ensures thread-safe initialization
  let _ = JS_HTTP_CLIENT.set(client);
  HTTP_CLIENT_REGISTERED.store(true, Ordering::SeqCst);
}

pub fn create_http_uri_plugin(
  _allowed_uris: Option<Vec<String>>,
  cache_location: Option<String>,
  frozen: Option<bool>,
  lockfile_location: Option<String>,
  proxy: Option<String>,
  upgrade: Option<bool>,
  filesystem: Arc<dyn WritableFileSystem>,
) -> Result<HttpUriPlugin, AnyhowError> {
  let allowed_uris = HttpUriOptionsAllowedUris::default();

  let http_client = if HTTP_CLIENT_REGISTERED.load(Ordering::SeqCst) {
    // Get a reference to the JS_HTTP_CLIENT using get() which returns Option<&T>
    // Then clone it to create a new instance
    let js_client = JS_HTTP_CLIENT
      .get()
      .expect("JS_HTTP_CLIENT was registered but is not initialized")
      .clone();

    Some(Arc::new(js_client) as Arc<dyn HttpClient>)
  } else {
    Some(Arc::new(SimpleHttpClient) as Arc<dyn HttpClient>)
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

#[derive(Debug)]
pub struct SimpleHttpClient;

#[async_trait]
impl HttpClient for SimpleHttpClient {
  async fn get(
    &self,
    _url: &str,
    headers: &HashMap<String, String>,
  ) -> anyhow::Result<HttpResponse> {
    let response = HttpResponse {
      status: 200,
      headers: headers.clone(),
      body: vec![],
    };

    Ok(response)
  }
}
