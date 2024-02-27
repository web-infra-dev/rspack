use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi(object)]
pub struct JsHooks {
  pub process_assets_stage_additional: JsFunction,
  pub process_assets_stage_pre_process: JsFunction,
  pub process_assets_stage_derived: JsFunction,
  pub process_assets_stage_additions: JsFunction,
  pub process_assets_stage_none: JsFunction,
  pub process_assets_stage_optimize: JsFunction,
  pub process_assets_stage_optimize_count: JsFunction,
  pub process_assets_stage_optimize_compatibility: JsFunction,
  pub process_assets_stage_optimize_size: JsFunction,
  pub process_assets_stage_dev_tooling: JsFunction,
  pub process_assets_stage_optimize_inline: JsFunction,
  pub process_assets_stage_summarize: JsFunction,
  pub process_assets_stage_optimize_hash: JsFunction,
  pub process_assets_stage_optimize_transfer: JsFunction,
  pub process_assets_stage_analyse: JsFunction,
  pub process_assets_stage_report: JsFunction,
  pub after_process_assets: JsFunction,
  pub this_compilation: JsFunction,
  pub emit: JsFunction,
  pub asset_emitted: JsFunction,
  pub should_emit: JsFunction,
  pub after_emit: JsFunction,
  pub optimize_modules: JsFunction,
  pub after_optimize_modules: JsFunction,
  pub optimize_tree: JsFunction,
  pub optimize_chunk_modules: JsFunction,
  pub before_compile: JsFunction,
  pub after_compile: JsFunction,
  pub finish_modules: JsFunction,
  pub finish_make: JsFunction,
  pub build_module: JsFunction,
  pub before_resolve: JsFunction,
  pub after_resolve: JsFunction,
  pub context_module_factory_before_resolve: JsFunction,
  pub normal_module_factory_create_module: JsFunction,
  pub normal_module_factory_resolve_for_scheme: JsFunction,
  pub chunk_asset: JsFunction,
  pub succeed_module: JsFunction,
  pub still_valid_module: JsFunction,
  pub execute_module: JsFunction,
  pub runtime_module: JsFunction,
}

#[napi(string_enum)]
#[derive(Debug)]
pub enum JsHookType {
  CompilerCompilation,
  CompilerMake,
}

#[napi(object)]
pub struct JsHook {
  pub r#type: JsHookType,
  pub function: JsFunction,
}
