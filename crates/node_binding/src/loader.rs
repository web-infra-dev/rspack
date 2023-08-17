use napi::Result;
use rspack_binding_options::{run_builtin_loader as run_builtin, JsLoaderContext};

/// Builtin loader runner
#[napi(catch_unwind)]
#[allow(unused)]
pub async fn run_builtin_loader(
  builtin: String,
  options: Option<&str>,
  loader_context: JsLoaderContext,
) -> Result<JsLoaderContext> {
  run_builtin(builtin, options, loader_context).await
}
