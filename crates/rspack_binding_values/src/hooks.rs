use napi_derive::napi;
use rspack_napi::threadsafe_function::ThreadsafeFunction;

use crate::{
  AfterResolveCreateData, AfterResolveData, CreateModuleData, JsBeforeResolveArgs,
  JsResolveForSchemeInput, JsResolveForSchemeResult,
};

#[napi(object, object_to_js = false)]
pub struct JsHooks {
  #[napi(
    ts_type = "(data: AfterResolveData) => Promise<(boolean | void | AfterResolveCreateData)[]>"
  )]
  pub after_resolve:
    ThreadsafeFunction<AfterResolveData, (Option<bool>, Option<AfterResolveCreateData>)>,
  #[napi(ts_type = "(data: JsBeforeResolveArgs) => Promise<boolean | void>")]
  pub context_module_factory_before_resolve: ThreadsafeFunction<JsBeforeResolveArgs, Option<bool>>,
  #[napi(ts_type = "(data: AfterResolveData) => Promise<boolean | void>")]
  pub context_module_factory_after_resolve: ThreadsafeFunction<AfterResolveData, Option<bool>>,
  #[napi(ts_type = "(data: CreateModuleData) => void")]
  pub normal_module_factory_create_module: ThreadsafeFunction<CreateModuleData, ()>,
  #[napi(ts_type = "(data: JsResolveForSchemeInput) => Promise<JsResolveForSchemeResult>")]
  pub normal_module_factory_resolve_for_scheme:
    ThreadsafeFunction<JsResolveForSchemeInput, JsResolveForSchemeResult>,
}
