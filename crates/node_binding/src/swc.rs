use rspack_javascript_compiler::{
  minify::JsMinifyOptions, transform::SwcOptions, JavaScriptCompiler,
  TransformOutput as CompilerTransformOutput,
};
use swc_core::ecma::ast::noop_pass;

#[napi(object)]
pub struct TransformOutput {
  pub code: String,
  pub map: Option<String>,
  pub diagnostics: Vec<String>,
}

impl From<CompilerTransformOutput> for TransformOutput {
  fn from(value: CompilerTransformOutput) -> Self {
    Self {
      code: value.code,
      map: value
        .map
        .map(|v| serde_json::to_string(&v).expect("failed to serialize transformOutput.map")),
      diagnostics: value.diagnostics,
    }
  }
}

fn _transform(source: String, options: String) -> napi::Result<TransformOutput> {
  let options: SwcOptions = serde_json::from_str(&options)?;
  let compiler = JavaScriptCompiler::new();
  compiler
    .transform(
      source,
      Some(swc_core::common::FileName::Anon),
      options,
      None,
      |_| noop_pass(),
    )
    .map(TransformOutput::from)
    .map_err(|e| napi::Error::new(napi::Status::GenericFailure, format!("{e}")))
}

#[napi]
pub async fn transform(source: String, options: String) -> napi::Result<TransformOutput> {
  _transform(source, options)
}

#[napi]
pub fn transform_sync(source: String, options: String) -> napi::Result<TransformOutput> {
  let future = async move { _transform(source, options) };
  // Fork from https://github.com/swc-project/swc/blob/main/crates/swc/src/plugin.rs#L76-L81
  // Ensure swc wasm plugin can get async contexts from tokio.
  if let Ok(handle) = tokio::runtime::Handle::try_current() {
    handle.block_on(future)
  } else {
    tokio::runtime::Runtime::new()
      .map_err(|e| napi::Error::new(napi::Status::GenericFailure, e))?
      .block_on(future)
  }
}

fn _minify(source: String, options: String) -> napi::Result<TransformOutput> {
  let options: JsMinifyOptions = serde_json::from_str(&options)?;
  let compiler = JavaScriptCompiler::new();
  compiler
    .minify(
      swc_core::common::FileName::Anon,
      source,
      options,
      None::<&dyn Fn(&swc_core::common::comments::SingleThreadedComments)>,
    )
    .map(TransformOutput::from)
    .map_err(|e| {
      let v = e.into_inner();
      let err = v
        .into_iter()
        .map(|e| format!("{e:?}"))
        .collect::<Vec<_>>()
        .join("\n");

      napi::Error::new(napi::Status::GenericFailure, err)
    })
}

#[napi]
pub async fn minify(source: String, options: String) -> napi::Result<TransformOutput> {
  _minify(source, options)
}

#[napi]
pub fn minify_sync(source: String, options: String) -> napi::Result<TransformOutput> {
  _minify(source, options)
}
