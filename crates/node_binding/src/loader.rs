use napi::Result;
use rspack_binding_options::JsLoaderContext;

/// Builtin loader runner
#[napi]
pub async fn run_builtin_loader(
  builtin: String,
  options: Option<String>,
  loader_context: JsLoaderContext,
) -> Result<JsLoaderContext> {
  rspack_binding_options::run_builtin_loader(builtin, options.as_deref(), loader_context).await
}
