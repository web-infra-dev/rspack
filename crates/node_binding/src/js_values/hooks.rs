use napi::bindgen_prelude::*;

#[napi(object)]
pub struct JsHooks {
  pub process_assets_stage_additional: JsFunction,
  pub process_assets_stage_pre_process: JsFunction,
  pub process_assets_stage_none: JsFunction,
  pub process_assets_stage_summarize: JsFunction,
  pub compilation: JsFunction,
  pub this_compilation: JsFunction,
  pub emit: JsFunction,
  pub after_emit: JsFunction,
}
