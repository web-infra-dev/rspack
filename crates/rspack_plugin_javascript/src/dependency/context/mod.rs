mod amd_require_context_dependency;
mod common_js_require_context_dependency;
mod import_context_dependency;
mod import_meta_context_dependency;
mod require_context_dependency;
mod require_resolve_context_dependency;

pub use amd_require_context_dependency::AMDRequireContextDependency;
pub use common_js_require_context_dependency::CommonJsRequireContextDependency;
pub use import_context_dependency::ImportContextDependency;
pub use import_meta_context_dependency::ImportMetaContextDependency;
use itertools::Itertools;
pub use require_context_dependency::RequireContextDependency;
pub use require_resolve_context_dependency::RequireResolveContextDependency;
use rspack_core::{
  module_raw, ContextDependency, ContextMode, ContextOptions, DependencyRange, GroupOptions,
  TemplateContext, TemplateReplaceSource,
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
    .map(|r| r.to_source_string())
    .unwrap_or_default();
  let include = options
    .include
    .as_ref()
    .map(|x| x.to_source_string())
    .unwrap_or_default();
  let exclude = options
    .exclude
    .as_ref()
    .map(|x| x.to_source_string())
    .unwrap_or_default();
  let mode = options.mode.as_str();
  let referenced_exports = options
    .referenced_exports
    .as_ref()
    .map(|x| x.iter().map(|x| format!(r#""{x}""#)).join(","))
    .unwrap_or_default();
  let mut group_options = String::new();

  if let Some(GroupOptions::ChunkGroup(group)) = &options.group_options {
    if let Some(chunk_name) = &group.name {
      group_options += chunk_name;
    }
    group_options += " {";
    if let Some(o) = group.prefetch_order {
      group_options.push_str(&format!("prefetchOrder: {},", o));
    }
    if let Some(o) = group.preload_order {
      group_options.push_str(&format!("preloadOrder: {},", o));
    }
    if let Some(o) = group.fetch_priority {
      group_options.push_str(&format!("fetchPriority: {},", o));
    }
    group_options += "}";
  }

  let id = format!(
    "context{context}|ctx request{request} {recursive} {regexp} {include} {exclude} {mode} {group_options} {referenced_exports}"
  );
  id
}

fn context_dependency_template_as_require_call(
  dep: &dyn ContextDependency,
  source: &mut TemplateReplaceSource,
  code_generatable_context: &mut TemplateContext,
  range: &DependencyRange,
  value_range: Option<&DependencyRange>,
) {
  let TemplateContext {
    compilation,
    runtime_requirements,
    ..
  } = code_generatable_context;
  let id = dep.id();

  let mut expr = module_raw(compilation, runtime_requirements, id, dep.request(), false);

  if compilation
    .get_module_graph()
    .module_graph_module_by_dependency_id(id)
    .is_some()
    && let Some(value_range) = value_range
  {
    for (content, start, end) in &dep.options().replaces {
      source.replace(*start, *end, content, None);
    }
    source.replace(value_range.end, range.end, ")", None);
    expr.push('(');
    source.replace(range.start, value_range.start, &expr, None);
    return;
  }
  source.replace(range.start, range.end, &expr, None);
}

fn context_dependency_template_as_id(
  dep: &dyn ContextDependency,
  source: &mut TemplateReplaceSource,
  code_generatable_context: &mut TemplateContext,
  range: &DependencyRange,
) {
  let TemplateContext {
    compilation,
    runtime_requirements,
    ..
  } = code_generatable_context;
  let id = dep.id();

  let expr = module_raw(
    compilation,
    runtime_requirements,
    id,
    dep.request(),
    dep.options().mode == ContextMode::Weak,
  );

  if compilation
    .get_module_graph()
    .module_graph_module_by_dependency_id(id)
    .is_none()
  {
    source.replace(range.start, range.end, &expr, None);
    return;
  }

  for (content, start, end) in &dep.options().replaces {
    source.replace(*start, *end, content, None);
  }

  source.replace(
    range.start,
    range.start,
    &format!("{}.resolve(", &expr),
    None,
  );
  source.replace(range.end, range.end, ")", None);
}
