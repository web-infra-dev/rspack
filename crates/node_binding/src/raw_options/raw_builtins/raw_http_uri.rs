use std::{
  collections::HashMap,
  fmt::Debug,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
  },
};

use async_trait::async_trait;
use napi::{
  bindgen_prelude::{Buffer, Function},
  Env, JsUnknown, NapiRaw,
};
use napi_derive::napi;
use rspack_error::AnyhowError;
use rspack_fs::WritableFileSystem;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_plugin_schemes::{
  http_cache::HttpResponse, HttpClient, HttpUriOptionsAllowedUris, HttpUriPlugin,
  HttpUriPluginOptions,
};

// Flag indicating if HTTP client is registered
static HTTP_CLIENT_REGISTERED: AtomicBool = AtomicBool::new(false);
// Thread-safe storage for the ThreadsafeFunction
static mut HTTP_CLIENT_FUNCTION: Option<
  ThreadsafeFunction<(String, HashMap<String, String>), Buffer>,
> = None;

// Implementation of HttpClient that bridges to JavaScript
#[derive(Debug)]
pub struct JsHttpClient;

#[async_trait]
impl HttpClient for JsHttpClient {
  async fn get(
    &self,
    url: &str,
    headers: &HashMap<String, String>,
  ) -> anyhow::Result<HttpResponse> {
    // Check if a client is registered
    if !HTTP_CLIENT_REGISTERED.load(Ordering::SeqCst) {
      return Err(anyhow::anyhow!("HTTP client function not registered"));
    }

    // Get the ThreadsafeFunction - unsafe is necessary here as we're accessing static mut
    let tsfn = unsafe {
      match &HTTP_CLIENT_FUNCTION {
        Some(f) => f.clone(),
        None => return Err(anyhow::anyhow!("HTTP client function not initialized")),
      }
    };

    // Clone the values for the async call
    let url_owned = url.to_string();
    let headers_owned = headers.clone();

    // Call the JavaScript function with both arguments in a tuple
    match tsfn.call_with_sync((url_owned, headers_owned)).await {
      Ok(buffer) => {
        // Create response with the returned buffer
        let response = HttpResponse {
          status: 200, // Default status
          headers: headers.clone(),
          body: buffer.to_vec(),
        };
        Ok(response)
      }
      Err(e) => Err(anyhow::anyhow!("JS HTTP request failed: {}", e)),
    }
  }
}

// Register the HTTP client function from JavaScript
#[napi]
pub fn register_http_client(http_client: Function) -> napi::Result<()> {
  // Convert the JS function to a ThreadsafeFunction
  let env = http_client.env;
  let tsfn = unsafe {
    ThreadsafeFunction::<(String, HashMap<String, String>), Buffer>::from_napi_value(
      env.raw(),
      http_client.raw(),
    )?
  };

  // Store the ThreadsafeFunction in our global static
  unsafe {
    HTTP_CLIENT_FUNCTION = Some(tsfn);
  }

  // Mark that client is registered
  HTTP_CLIENT_REGISTERED.store(true, Ordering::SeqCst);
  Ok(())
}

// Create a new HttpUriPlugin
pub fn create_http_uri_plugin(
  _allowed_uris: Option<Vec<String>>,
  cache_location: Option<String>,
  frozen: Option<bool>,
  lockfile_location: Option<String>,
  proxy: Option<String>,
  upgrade: Option<bool>,
  filesystem: Arc<dyn WritableFileSystem>,
) -> Result<HttpUriPlugin, AnyhowError> {
  // Create allowed_uris using default - this struct has no fields
  let allowed_uris = HttpUriOptionsAllowedUris::default();

  // Create an HTTP client instance
  let http_client = Some(Arc::new(JsHttpClient) as Arc<dyn HttpClient>);

  // Create plugin options
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

  // Create and return the plugin
  Ok(HttpUriPlugin::new(options))
}
