use rspack_javascript_compiler::{
  minify::JsMinifyOptions, transform::SwcOptions, JavaScriptCompiler,
  TransformOutput as CompilerTransformOutput,
};
use swc_core::ecma::ast::noop_pass;

#[napi(object)]
pub struct TransformOutput {
  pub code: String,
  pub map: Option<String>,
}

impl From<CompilerTransformOutput> for TransformOutput {
  fn from(value: CompilerTransformOutput) -> Self {
    Self {
      code: value.code,
      map: value
        .map
        .map(|v| serde_json::to_string(&v).expect("failed to serialize transformOutput.map")),
    }
  }
}

#[napi]
pub async fn transform(source: String, options: String) -> napi::Result<TransformOutput> {
  let options: SwcOptions = serde_json::from_str(&options)?;
  let javascropt_compiler = JavaScriptCompiler::new();
  javascropt_compiler
    .transform(
      source,
      Some(swc_core::common::FileName::Anon),
      options,
      None,
      |_| noop_pass(),
    )
    .map(|o| o.into())
    .map_err(|e| napi::Error::new(napi::Status::GenericFailure, format!("{}", e)))
}

#[napi]
pub async fn minify(source: String, options: String) -> napi::Result<TransformOutput> {
  let options: JsMinifyOptions = serde_json::from_str(&options)?;
  let javascropt_compiler = JavaScriptCompiler::new();
  javascropt_compiler
    .minify(
      swc_core::common::FileName::Anon,
      source,
      options,
      None::<&dyn Fn(&swc_core::common::comments::SingleThreadedComments)>,
    )
    .map(|o| o.into())
    .map_err(|e| {
      let v = e.into_inner();
      let err = v
        .into_iter()
        .map(|e| format!("{:?}", e))
        .collect::<Vec<_>>()
        .join("\n");

      napi::Error::new(napi::Status::GenericFailure, format!("{err}"))
    })
}
