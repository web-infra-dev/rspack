use napi::bindgen_prelude::Buffer;
use napi_derive::napi;

#[napi(object, object_from_js = false)]
pub struct JsUpdateHashData {
  pub assets: Vec<Buffer>,
  pub old_hash: String,
}
