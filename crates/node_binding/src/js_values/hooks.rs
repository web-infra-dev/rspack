use napi::bindgen_prelude::*;

#[napi(object)]
pub struct JsHooks {
  pub process_assets_stage_additional: JsFunction,
  pub process_assets_stage_pre_process: JsFunction,
  pub process_assets_stage_additions: JsFunction,
  pub process_assets_stage_none: JsFunction,
  pub process_assets_stage_optimize_inline: JsFunction,
  pub process_assets_stage_summarize: JsFunction,
  pub process_assets_stage_optimize_hash: JsFunction,
  pub process_assets_stage_report: JsFunction,
  pub compilation: JsFunction,
  pub this_compilation: JsFunction,
  pub emit: JsFunction,
  pub after_emit: JsFunction,
  pub make: JsFunction,
  pub optimize_modules: JsFunction,
  pub optimize_chunk_module: JsFunction,
  pub before_compile: JsFunction,
  pub after_compile: JsFunction,
  pub finish_modules: JsFunction,
  pub before_resolve: JsFunction,
  pub after_resolve: JsFunction,
  pub context_module_before_resolve: JsFunction,
  pub normal_module_factory_resolve_for_scheme: JsFunction,
  pub chunk_asset: JsFunction,
}
