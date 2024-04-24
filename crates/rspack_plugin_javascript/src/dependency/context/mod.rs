mod common_js_require_context_dependency;
mod import_context_dependency;
mod import_meta_context_dependency;
mod require_context_dependency;

pub use common_js_require_context_dependency::CommonJsRequireContextDependency;
pub use import_context_dependency::ImportContextDependency;
pub use import_meta_context_dependency::ImportMetaContextDependency;
pub use require_context_dependency::RequireContextDependency;
use rspack_core::{
  module_raw, ContextDependency, ContextOptions, TemplateContext, TemplateReplaceSource,
};

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

fn context_dependency_template_as_require_call(
  dep: &dyn ContextDependency,
  source: &mut TemplateReplaceSource,
  code_generatable_context: &mut TemplateContext,
  callee_start: u32,
  callee_end: u32,
  args_end: u32,
) {
  let TemplateContext {
    compilation,
    runtime_requirements,
    ..
  } = code_generatable_context;
  let id = dep.id();

  let expr = module_raw(compilation, runtime_requirements, id, dep.request(), false);

  if compilation
    .get_module_graph()
    .module_graph_module_by_dependency_id(id)
    .is_none()
  {
    source.replace(callee_start, args_end, &expr, None);
    return;
  }

  for (content, start, end) in &dep.options().replaces {
    source.replace(*start, *end - 1, content, None);
  }
  source.replace(callee_start, callee_end, &expr, None);
}
