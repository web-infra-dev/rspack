use std::path::Path;
use std::sync::Arc;

use napi::Either;
use napi_derive::napi;
use rspack_core::Resolver;

#[napi]
pub struct JsResolver {
  inner: Arc<Resolver>,
}

type ResolveResult = napi::Either<String, bool>;

impl JsResolver {
  pub(crate) fn new(resolver: Arc<Resolver>) -> Self {
    Self { inner: resolver }
  }
}

#[napi]
impl JsResolver {
  #[napi(ts_return_type = "string | false")]
  pub fn resolve(&self, path: String, request: String) -> ResolveResult {
    match self.inner.resolve(Path::new(&path), &request) {
      Ok(rspack_core::ResolveResult::Resource(resource)) => {
        Either::A(resource.full_path().to_string_lossy().to_string())
      }
      Ok(rspack_core::ResolveResult::Ignored) => Either::B(false),
      Err(err) => Either::A(format!("error:{:#?}", err)),
    }
  }
}
