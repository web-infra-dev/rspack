use std::{
  collections::HashMap,
  fmt::Debug,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
  },
};

use async_trait::async_trait;
use napi::bindgen_prelude::{Buffer, Function};
use napi_derive::napi;
use rspack_error::AnyhowError;
use rspack_fs::WritableFileSystem;
use rspack_plugin_schemes::{
  HttpClient, HttpResponse, HttpUriOptionsAllowedUris, HttpUriPlugin, HttpUriPluginOptions,
};

static HTTP_CLIENT_REGISTERED: AtomicBool = AtomicBool::new(false);
static mut HTTP_CLIENT_FUNCTION: Option<Arc<Mutex<Function>>> = None;

#[derive(Debug)]
pub struct JsHttpClient;

#[async_trait]
impl HttpClient for JsHttpClient {
  async fn get(
    &self,
    url: &str,
    headers: &HashMap<String, String>,
  ) -> anyhow::Result<HttpResponse> {
    if !HTTP_CLIENT_REGISTERED.load(Ordering::SeqCst) {
      return Err(anyhow::anyhow!("HTTP client function not registered"));
    }

    let func = unsafe {
      match &HTTP_CLIENT_FUNCTION {
        Some(f) => f.clone(),
        None => return Err(anyhow::anyhow!("HTTP client function not registered")),
      }
    };

    let (sender, receiver) = tokio::sync::oneshot::channel();

    let url_owned = url.to_owned();
    let headers_owned = headers.clone();

    tokio::task::spawn_blocking(move || {
      let func = func.lock().unwrap();

      let callback = move |result: Result<napi::Either<Buffer, String>, napi::Error>| {
        match result {
          Ok(napi::Either::A(buffer)) => {
            let body = buffer.to_vec();

            let response = HttpResponse {
              status: 200,
              headers: headers_owned,
              body,
            };

            let _ = sender.send(Ok(response));
          }
          Ok(napi::Either::B(error_str)) => {
            let _ = sender.send(Err(anyhow::anyhow!(error_str)));
          }
          Err(e) => {
            let _ = sender.send(Err(anyhow::anyhow!("JavaScript HTTP client error: {}", e)));
          }
        }
      };

      match func.call(url_owned, headers_owned) {
        Ok(_) => {}
        Err(e) => {
          let _ = sender.send(Err(anyhow::anyhow!(
            "Failed to call JavaScript HTTP client: {}",
            e
          )));
        }
      }
    });

    match receiver.await {
      Ok(result) => result,
      Err(e) => Err(anyhow::anyhow!("Failed to receive HTTP response: {}", e)),
    }
  }
}

#[napi]
pub fn register_http_client(http_client: Function) -> napi::Result<()> {
  unsafe {
    HTTP_CLIENT_FUNCTION = Some(Arc::new(Mutex::new(http_client)));
  }

  HTTP_CLIENT_REGISTERED.store(true, Ordering::SeqCst);
  Ok(())
}

pub fn create_http_uri_plugin(
  allowed_uris: Option<Vec<String>>,
  cache_location: Option<String>,
  frozen: Option<bool>,
  lockfile_location: Option<String>,
  proxy: Option<String>,
  upgrade: Option<bool>,
  filesystem: Arc<dyn WritableFileSystem>,
) -> Result<HttpUriPlugin, AnyhowError> {
  let allowed_uris = HttpUriOptionsAllowedUris::default();

  let http_client = Some(Arc::new(JsHttpClient) as Arc<dyn HttpClient>);

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
