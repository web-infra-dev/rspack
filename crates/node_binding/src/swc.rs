use napi::{bindgen_prelude::AsyncTask, Task};
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

pub struct AsyncTransform {
  source: String,
  options: String,
}

impl Task for AsyncTransform {
  type Output = CompilerTransformOutput;
  type JsValue = TransformOutput;

  fn compute(&mut self) -> napi::Result<Self::Output> {
    let compiler = JavaScriptCompiler::new();
    let options: SwcOptions = serde_json::from_str(&self.options)?;
    let source = std::mem::take(&mut self.source);
    compiler
      .transform(
        source,
        Some(swc_core::common::FileName::Anon),
        options,
        None,
        |_| noop_pass(),
      )
      .map_err(|e| napi::Error::new(napi::Status::GenericFailure, format!("{}", e)))
  }

  fn resolve(&mut self, _env: napi::Env, output: Self::Output) -> napi::Result<Self::JsValue> {
    let output = TransformOutput::from(output);
    Ok(output)
  }
}

pub struct AsyncMinifier {
  source: String,
  options: String,
}

impl Task for AsyncMinifier {
  type Output = CompilerTransformOutput;
  type JsValue = TransformOutput;

  fn compute(&mut self) -> napi::Result<Self::Output> {
    let options: JsMinifyOptions = serde_json::from_str(&self.options)?;
    let compiler = JavaScriptCompiler::new();
    let source = std::mem::take(&mut self.source);
    compiler
      .minify(
        swc_core::common::FileName::Anon,
        source,
        options,
        None::<&dyn Fn(&swc_core::common::comments::SingleThreadedComments)>,
      )
      .map_err(|e| {
        let v = e.into_inner();
        let err = v
          .into_iter()
          .map(|e| format!("{:?}", e))
          .collect::<Vec<_>>()
          .join("\n");

        napi::Error::new(napi::Status::GenericFailure, err)
      })
  }

  fn resolve(&mut self, _env: napi::Env, output: Self::Output) -> napi::Result<Self::JsValue> {
    let output = TransformOutput::from(output);
    Ok(output)
  }
}

#[napi(ts_return_type = "Promise<TransformOutput>")]
pub fn transform(source: String, options: String) -> AsyncTask<AsyncTransform> {
  AsyncTask::new(AsyncTransform { source, options })
}

#[napi(ts_return_type = "Promise<TransformOutput>")]
pub fn minify(source: String, options: String) -> AsyncTask<AsyncMinifier> {
  AsyncTask::new(AsyncMinifier { source, options })
}
