mod amd_require_context_dependency;
mod common_js_require_context_dependency;
mod import_context_dependency;
mod import_meta_context_dependency;
mod require_context_dependency;
mod require_resolve_context_dependency;
mod url_context_dependency;

pub use amd_require_context_dependency::{
  AMDRequireContextDependency, AMDRequireContextDependencyTemplate,
};
pub use common_js_require_context_dependency::{
  CommonJsRequireContextDependency, CommonJsRequireContextDependencyTemplate,
};
pub use import_context_dependency::{ImportContextDependency, ImportContextDependencyTemplate};
pub use import_meta_context_dependency::{
  ImportMetaContextDependency, ImportMetaContextDependencyTemplate,
};
use itertools::Itertools;
pub use require_context_dependency::{RequireContextDependency, RequireContextDependencyTemplate};
pub use require_resolve_context_dependency::{
  RequireResolveContextDependency, RequireResolveContextDependencyTemplate,
};
use rspack_core::{
  ContextDependency, ContextMode, ContextOptions, DependencyRange, GroupOptions,
  ResourceIdentifier, TemplateContext, TemplateReplaceSource,
};
pub use url_context_dependency::{URLContextDependency, URLContextDependencyTemplate};

fn create_resource_identifier_for_context_dependency(
  context: Option<&str>,
  options: &ContextOptions,
) -> ResourceIdentifier {
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
    .map(|ids| ids.iter().map(|ids| ids.iter().join(".")).join(", "))
    .unwrap_or_default();
  let mut group_options = String::new();

  if let Some(GroupOptions::ChunkGroup(group)) = &options.group_options {
    if let Some(chunk_name) = &group.name {
      group_options += chunk_name;
    }
    group_options += " {";
    if let Some(o) = group.prefetch_order {
      group_options.push_str(&format!("prefetchOrder: {o},"));
    }
    if let Some(o) = group.preload_order {
      group_options.push_str(&format!("preloadOrder: {o},"));
    }
    if let Some(o) = group.fetch_priority {
      group_options.push_str(&format!("fetchPriority: {o},"));
    }
    group_options += "}";
  }

  let id = format!(
    "context{context}|ctx request{request} {recursive} {regexp} {include} {exclude} {mode} {group_options} {referenced_exports}"
  );
  id.into()
}

fn context_dependency_template_as_require_call(
  dep: &dyn ContextDependency,
  source: &mut TemplateReplaceSource,
  code_generatable_context: &mut TemplateContext,
  range: DependencyRange,
  value_range: Option<&DependencyRange>,
) {
  let TemplateContext {
    compilation,
    runtime_template,
    ..
  } = code_generatable_context;
  let id = dep.id();

  let mut expr = runtime_template.module_raw(compilation, id, dep.request(), false);

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
  range: DependencyRange,
) {
  let TemplateContext {
    compilation,
    runtime_template,
    ..
  } = code_generatable_context;
  let id = dep.id();

  let expr = runtime_template.module_raw(
    compilation,
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
