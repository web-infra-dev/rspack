use napi_derive::napi;
use rspack_napi_shared::new_tsfn::ThreadsafeFunction;

use crate::{
  AfterResolveData, BeforeResolveData, CreateModuleData, JsAssetEmittedArgs, JsChunkAssetArgs,
  JsCompilation, JsExecuteModuleArg, JsModule, JsResolveForSchemeInput, JsResolveForSchemeResult,
  JsRuntimeModule, JsRuntimeModuleArg,
};

#[napi(object, object_to_js = false)]
pub struct JsHooks {
  #[napi(ts_type = "(compilation: JsCompilation) => void")]
  pub this_compilation: ThreadsafeFunction<JsCompilation, ()>,
  #[napi(ts_type = "() => void")]
  pub after_process_assets: ThreadsafeFunction<(), ()>,
  #[napi(ts_type = "() => void")]
  pub emit: ThreadsafeFunction<(), ()>,
  #[napi(ts_type = "(asset: JsAssetEmittedArgs) => void")]
  pub asset_emitted: ThreadsafeFunction<JsAssetEmittedArgs, ()>,
  #[napi(ts_type = "(compilation: JsCompilation) => void")]
  pub should_emit: ThreadsafeFunction<JsCompilation, Option<bool>>,
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
  #[napi(ts_type = "() => void")]
  pub before_compile: ThreadsafeFunction<(), ()>,
  #[napi(ts_type = "(compilation: JsCompilation) => void")]
  pub after_compile: ThreadsafeFunction<JsCompilation, ()>,
  #[napi(ts_type = "(compilation: JsCompilation) => void")]
  pub finish_modules: ThreadsafeFunction<JsCompilation, ()>,
  #[napi(ts_type = "(compilation: JsCompilation) => void")]
  pub finish_make: ThreadsafeFunction<JsCompilation, ()>,
  #[napi(ts_type = "(module: JsModule) => void")]
  pub build_module: ThreadsafeFunction<JsModule, ()>, // TODO
  #[napi(ts_type = "(asset: JsChunkAssetArgs) => void")]
  pub chunk_asset: ThreadsafeFunction<JsChunkAssetArgs, ()>,
  #[napi(ts_type = "(data: BeforeResolveData) => Promise<(boolean | void | BeforeResolveData)[]>")]
  pub before_resolve: ThreadsafeFunction<BeforeResolveData, (Option<bool>, BeforeResolveData)>,
  #[napi(ts_type = "(data: AfterResolveData) => Promise<boolean | void>")]
  pub after_resolve: ThreadsafeFunction<AfterResolveData, Option<bool>>,
  #[napi(ts_type = "(data: BeforeResolveData) => Promise<boolean | void>")]
  pub context_module_factory_before_resolve: ThreadsafeFunction<BeforeResolveData, Option<bool>>,
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
  #[napi(ts_type = "(arg: JsExecuteModuleArg) => void")]
  pub execute_module: ThreadsafeFunction<JsExecuteModuleArg, ()>,
  #[napi(ts_type = "(arg: JsRuntimeModuleArg) => JsRuntimeModule | void")]
  pub runtime_module: ThreadsafeFunction<JsRuntimeModuleArg, Option<JsRuntimeModule>>,
}
