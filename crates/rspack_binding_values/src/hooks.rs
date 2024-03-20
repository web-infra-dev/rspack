use napi_derive::napi;
use rspack_napi::threadsafe_function::ThreadsafeFunction;

use crate::{
  AfterResolveCreateData, AfterResolveData, CreateModuleData, JsAssetEmittedArgs,
  JsBeforeResolveArgs, JsChunkAssetArgs, JsCompilation, JsModule, JsResolveForSchemeInput,
  JsResolveForSchemeResult,
};

#[napi(object, object_to_js = false)]
pub struct JsHooks {
  #[napi(ts_type = "() => void")]
  pub after_process_assets: ThreadsafeFunction<(), ()>,
  #[napi(ts_type = "() => void")]
  pub emit: ThreadsafeFunction<(), ()>,
  #[napi(ts_type = "(asset: JsAssetEmittedArgs) => void")]
  pub asset_emitted: ThreadsafeFunction<JsAssetEmittedArgs, ()>,
  #[napi(ts_type = "() => void")]
  pub after_emit: ThreadsafeFunction<(), ()>,
  #[napi(ts_type = "(compilation: JsCompilation) => void")]
  pub optimize_modules: ThreadsafeFunction<JsCompilation, ()>,
  #[napi(ts_type = "(compilation: JsCompilation) => void")]
  pub after_optimize_modules: ThreadsafeFunction<JsCompilation, ()>,
  #[napi(ts_type = "() => void")]
  pub optimize_tree: ThreadsafeFunction<(), ()>,
  #[napi(ts_type = "(compilation: JsCompilation) => void")]
  pub optimize_chunk_modules: ThreadsafeFunction<JsCompilation, ()>,
  #[napi(ts_type = "(compilation: JsCompilation) => void")]
  pub finish_modules: ThreadsafeFunction<JsCompilation, ()>,
  #[napi(ts_type = "(compilation: JsCompilation) => void")]
  pub finish_make: ThreadsafeFunction<JsCompilation, ()>,
  #[napi(ts_type = "(module: JsModule) => void")]
  pub build_module: ThreadsafeFunction<JsModule, ()>, // TODO
  #[napi(ts_type = "(asset: JsChunkAssetArgs) => void")]
  pub chunk_asset: ThreadsafeFunction<JsChunkAssetArgs, ()>,
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
  #[napi(ts_type = "(module: JsModule) => void")]
  pub succeed_module: ThreadsafeFunction<JsModule, ()>,
  #[napi(ts_type = "(module: JsModule) => void")]
  pub still_valid_module: ThreadsafeFunction<JsModule, ()>,
}
