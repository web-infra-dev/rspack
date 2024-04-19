mod common_js_require_context_dependency;
mod import_context_dependency;
mod import_meta_context_dependency;
mod require_context_dependency;

pub use common_js_require_context_dependency::CommonJsRequireContextDependency;
pub use import_context_dependency::ImportContextDependency;
pub use import_meta_context_dependency::ImportMetaContextDependency;
pub use require_context_dependency::RequireContextDependency;
use rspack_core::ContextOptions;

fn create_resource_identifier_for_context_dependency(
  context: Option<&str>,
  options: &ContextOptions,
) -> String {
  let context = context.unwrap_or_default();
  let request = &options.request;
  let recursive = options.recursive.to_string();
  let regexp = options
    .reg_exp
    .as_ref()
    .map(|r| r.to_string())
    .unwrap_or_default();
  let include = options.include.as_deref().unwrap_or_default();
  let exclude = options.exclude.as_deref().unwrap_or_default();
  let mode = options.mode.as_str();
  // TODO: need `RawChunkGroupOptions`
  let id = format!(
    "context{context}|ctx request{request} {recursive} `{regexp} {include} {exclude} ``{mode} `"
  );
  id
}
