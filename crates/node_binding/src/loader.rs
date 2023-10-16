use napi::Result;
use rspack_binding_options::{run_builtin_loader as run_builtin, JsLoaderContext};

/// Builtin loader runner
#[napi(catch_unwind)]
#[allow(unused)]
pub async fn run_builtin_loader(
  builtin: String,
  options: Option<String>,
  loader_context: JsLoaderContext,
) -> Result<JsLoaderContext> {
  run_builtin(builtin, options.as_deref(), loader_context).await
}
