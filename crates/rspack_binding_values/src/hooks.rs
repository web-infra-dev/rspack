use napi_derive::napi;
use rspack_napi::threadsafe_function::ThreadsafeFunction;

use crate::{
  CreateModuleData, JsAfterResolveData, JsResolveForSchemeInput, JsResolveForSchemeResult,
};

#[napi(object, object_to_js = false)]
pub struct JsHooks {
  #[napi(ts_type = "(data: JsAfterResolveData) => Promise<boolean | void>")]
  pub context_module_factory_after_resolve: ThreadsafeFunction<JsAfterResolveData, Option<bool>>,
  #[napi(ts_type = "(data: CreateModuleData) => void")]
  pub normal_module_factory_create_module: ThreadsafeFunction<CreateModuleData, ()>,
  #[napi(ts_type = "(data: JsResolveForSchemeInput) => Promise<JsResolveForSchemeResult>")]
  pub normal_module_factory_resolve_for_scheme:
    ThreadsafeFunction<JsResolveForSchemeInput, JsResolveForSchemeResult>,
}
