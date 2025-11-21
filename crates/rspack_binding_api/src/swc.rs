use napi::bindgen_prelude::within_runtime_if_available;
use rspack_javascript_compiler::{
  JavaScriptCompiler, TransformOutput as CompilerTransformOutput, minify::JsMinifyOptions,
  transform::SwcOptions,
};
use rspack_util::source_map::SourceMapKind;
use swc_core::{base::config::SourceMapsConfig, ecma::ast::noop_pass};

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

fn _to_source_map_kind(source_maps: Option<SourceMapsConfig>) -> SourceMapKind {
  match source_maps {
    Some(SourceMapsConfig::Str(s)) if s == "inline" => SourceMapKind::SourceMap,
    Some(SourceMapsConfig::Str(s)) => {
      // Explicitly handle non-"inline" string values for SourceMapsConfig::Str.
      // Currently, only "inline" is supported; other values are treated as empty.
      // Consider updating this match arm if additional string values are supported in the future.
      SourceMapKind::empty()
    }
    Some(SourceMapsConfig::Bool(true)) => SourceMapKind::SourceMap,
    Some(SourceMapsConfig::Bool(false)) => SourceMapKind::empty(),
    _ => SourceMapKind::empty(),
  }
}

fn _transform(source: String, options: String) -> napi::Result<TransformOutput> {
  #[cfg_attr(not(feature = "plugin"), allow(unused_mut))]
  let mut options: SwcOptions = serde_json::from_str(&options)?;

  #[cfg(feature = "plugin")]
  {
    options.runtime_options = options.runtime_options.plugin_runtime(std::sync::Arc::new(
      rspack_util::swc::runtime::WasmtimeRuntime,
    ));
  }

  let compiler = JavaScriptCompiler::new();
  let module_source_map_kind = _to_source_map_kind(options.source_maps.clone());

  compiler
    .transform(
      source,
      Some(swc_core::common::FileName::Real(
        options.filename.clone().into(),
      )),
      options,
      Some(module_source_map_kind),
      |_, _| {},
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
  within_runtime_if_available(|| _transform(source, options))
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
