use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;
use napi_derive::napi;
use rspack_fs::{NativeFileSystem, WritableFileSystem};
use rspack_plugin_schemes::{
  HttpClient, HttpResponse, HttpUriOptionsAllowedUris, HttpUriPluginOptions,
};

use crate::get_http_client;

// Simple HTTP client that doesn't actually use JavaScript but just errors
#[derive(Debug)]
struct DummyHttpClient;

#[async_trait]
impl HttpClient for DummyHttpClient {
  async fn get(&self, _url: &str, _headers: &HashMap<String, String>) -> Result<HttpResponse> {
    // Return an error indicating the JavaScript client is required
    Err(anyhow::anyhow!(
      "HTTP client not available. Please provide an httpClient in your webpack.config.js"
    ))
  }
}

#[napi(object)]
pub struct RawHttpUriPluginOptions {
  pub cache_location: Option<String>,
  pub frozen: Option<bool>,
  pub lockfile_location: Option<String>,
  pub proxy: Option<String>,
  pub upgrade: Option<bool>,
  pub http_client: Option<napi::JsUnknown>,
  pub allowed_uris: Option<napi::JsUnknown>,
}

impl From<RawHttpUriPluginOptions> for HttpUriPluginOptions {
  fn from(options: RawHttpUriPluginOptions) -> Self {
    // Try to get the HTTP client from the global registry first
    let http_client = get_http_client().or_else(|| {
      // If no global client is registered, use the dummy client
      Some(Arc::new(DummyHttpClient) as Arc<dyn HttpClient>)
    });

    Self {
      allowed_uris: HttpUriOptionsAllowedUris::default(),
      cache_location: options.cache_location,
      frozen: options.frozen,
      lockfile_location: options.lockfile_location,
      proxy: options.proxy,
      upgrade: options.upgrade,
      filesystem: Arc::new(NativeFileSystem::new(false)) as Arc<dyn WritableFileSystem>,
      http_client,
    }
  }
}
