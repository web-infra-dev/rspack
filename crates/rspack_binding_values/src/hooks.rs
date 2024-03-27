use napi_derive::napi;
use rspack_napi::threadsafe_function::ThreadsafeFunction;

use crate::{JsResolveForSchemeInput, JsResolveForSchemeResult};

#[napi(object, object_to_js = false)]
pub struct JsHooks {
  #[napi(ts_type = "(data: JsResolveForSchemeInput) => Promise<JsResolveForSchemeResult>")]
  pub normal_module_factory_resolve_for_scheme:
    ThreadsafeFunction<JsResolveForSchemeInput, JsResolveForSchemeResult>,
}
