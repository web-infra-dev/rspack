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
  bindgen_prelude::{Buffer, Promise},
  threadsafe_function::{ThreadsafeFunction as NapiThreadsafeFunction, ThreadsafeFunctionCallMode},
};
use napi_derive::napi;
use rspack_error::{AnyhowError, Error as RspackError};
use rspack_fs::WritableFileSystem;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_plugin_schemes::{
  HttpClient, HttpResponse, HttpUriOptionsAllowedUris, HttpUriPlugin, HttpUriPluginOptions,
};

// Define our response type for the JS -> Rust bridge
#[napi(object)]
pub struct JsHttpResponseRaw {
  pub status: u16,
  pub headers: HashMap<String, String>,
  pub body: Buffer,
}

// Thread-safe wrapper for the JS HTTP client function
#[derive(Debug, Clone)]
pub struct JsHttpClient {
  // ThreadsafeFunction wrapper matching the binding.d.ts definition
  function: ThreadsafeFunction<
    (
      Option<String>,
      Option<String>,
      String,
      HashMap<String, String>,
    ),
    Promise<JsHttpResponseRaw>,
  >,
}

// Implement the HttpClient trait for our JS bridge
#[async_trait]
impl HttpClient for JsHttpClient {
  async fn get(
    &self,
    url: &str,
    headers: &HashMap<String, String>,
  ) -> anyhow::Result<HttpResponse> {
    let url_owned = url.to_string();
    let headers_owned = headers.clone();

    // Add debug logging
    println!(
      "[JsHttpClient] Preparing to call JS with URL: '{}'",
      url_owned
    );
    println!("[JsHttpClient] Headers: {:?}", headers_owned);

    // Clone the function before using it in async context to avoid MutexGuard Send issues
    let func = self.function.clone();

    // Ensure we pass parameters in the correct order expected by TS definition
    // The correct signature is (err?: string, method?: string, url: string, headers: HashMap)
    let result = func
      .call_with_promise((
        None,                    // err
        Some("GET".to_string()), // method
        url_owned,               // url
        headers_owned,           // headers
      ))
      .await
      .map_err(|e| anyhow::anyhow!("Error calling JavaScript HTTP client: {}", e))?;

    // Add debug logging for response
    println!(
      "[JsHttpClient] Received response with status: {}",
      result.status
    );

    // Convert JS response to the expected format
    Ok(HttpResponse {
      status: result.status,
      headers: result.headers,
      body: result.body.to_vec(),
    })
  }
}

// A global flag to track if HTTP client has been registered
static HTTP_CLIENT_REGISTERED: AtomicBool = AtomicBool::new(false);
static mut JS_HTTP_CLIENT: Option<JsHttpClient> = None;

// Register a JS HTTP client function
#[napi]
pub fn register_http_client(
  http_client: ThreadsafeFunction<
    (
      Option<String>,
      Option<String>,
      String,
      HashMap<String, String>,
    ),
    Promise<JsHttpResponseRaw>,
  >,
) {
  // Create the JsHttpClient instance
  let client = JsHttpClient {
    function: http_client,
  };

  // Store it in our static variable
  unsafe {
    JS_HTTP_CLIENT = Some(client);
  }

  // Mark as registered
  HTTP_CLIENT_REGISTERED.store(true, Ordering::SeqCst);
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

  // Use the JS HTTP client if registered, otherwise use SimpleHttpClient
  let http_client = if HTTP_CLIENT_REGISTERED.load(Ordering::SeqCst) {
    unsafe {
      // Safe because we only set this when the boolean is true
      Some(Arc::new(JS_HTTP_CLIENT.clone().unwrap()) as Arc<dyn HttpClient>)
    }
  } else {
    // Fallback to a simple client
    Some(Arc::new(SimpleHttpClient) as Arc<dyn HttpClient>)
  };

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
// Simple implementation that always returns a successful response
// Used as fallback when no JS client is registered
#[derive(Debug)]
pub struct SimpleHttpClient;

#[async_trait]
impl HttpClient for SimpleHttpClient {
  async fn get(
    &self,
    _url: &str,
    headers: &HashMap<String, String>,
  ) -> anyhow::Result<HttpResponse> {
    // Just return a mock response that satisfies the API
    let response = HttpResponse {
      status: 200,
      headers: headers.clone(),
      body: vec![], // Empty body
    };

    Ok(response)
  }
}
