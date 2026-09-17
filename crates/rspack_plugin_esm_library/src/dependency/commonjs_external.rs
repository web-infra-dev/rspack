use std::sync::Arc;

use atomic_refcell::AtomicRefCell;
use rspack_cacheable::{cacheable, cacheable_dyn, with::Skip};
use rspack_core::{
  BuildModuleGraphArtifact, CodeGenerationDataItem, CommonJsExternalRequireKind, Dependency,
  DependencyCodeGeneration, DependencyId, DependencyRange, DependencyTemplate, TemplateContext,
  TemplateReplaceSource,
};
use rspack_plugin_javascript::dependency::{
  CommonJsExportRequireDependency, CommonJsExportRequireDependencyTemplate,
  CommonJsFullRequireDependency, CommonJsRequireDependency, RequireHeaderDependency,
};
use rspack_util::{
  fx_hash::{FxHashMap, FxHashSet},
  json_stringify_str,
};

use super::ExternalBindingBailout;

pub type DirectCommonJsExternalDependencies = Arc<AtomicRefCell<FxHashSet<DependencyId>>>;

#[cacheable]
#[derive(Debug, Clone)]
struct DirectExternalRequireHeaders(
  // Only shared between dependency templates during codegen. Cached sources
  // already contain the rendered callee and never consume these ranges again.
  #[cacheable(with=Skip)] Vec<(DependencyRange, CommonJsExternalRequireKind)>,
);

#[cacheable_dyn]
impl CodeGenerationDataItem for DirectExternalRequireHeaders {}

fn is_relative_external_request(request: &str) -> bool {
  request == "."
    || request == ".."
    || request.starts_with("./")
    || request.starts_with("../")
    || request.starts_with(".\\")
    || request.starts_with("..\\")
}

pub fn cutout_commonjs_externals(
  build_module_graph_artifact: &mut BuildModuleGraphArtifact,
) -> FxHashSet<DependencyId> {
  let module_graph = build_module_graph_artifact.get_module_graph();
  let mut direct_dependencies = FxHashSet::default();

  for (_, module) in module_graph.modules() {
    if ExternalBindingBailout::is_present(module.as_ref()) {
      continue;
    }
    let require_header_ranges = module
      .get_presentational_dependencies()
      .into_iter()
      .flatten()
      .filter_map(|dependency| {
        dependency
          .as_any()
          .downcast_ref::<RequireHeaderDependency>()
          .map(RequireHeaderDependency::range)
      })
      .collect::<Vec<_>>();
    let mut dependencies_by_range = FxHashMap::<DependencyRange, (DependencyId, usize)>::default();
    let mut self_rendering_require_dependencies = Vec::new();

    for dependency in module.get_dependencies() {
      let dependency = dependency.as_ref();
      let dependency_id = dependency.id();
      if dependency.as_any().is::<CommonJsFullRequireDependency>()
        || dependency.as_any().is::<CommonJsExportRequireDependency>()
      {
        self_rendering_require_dependencies.push(*dependency_id);
        continue;
      }
      let Some(dependency) = dependency
        .as_any()
        .downcast_ref::<CommonJsRequireDependency>()
      else {
        continue;
      };
      if dependency.get_context().is_some() {
        continue;
      }
      let Some(expression_range) = dependency.range() else {
        continue;
      };

      let entry = dependencies_by_range
        .entry(expression_range)
        .or_insert((*dependency_id, 0));
      entry.1 += 1;
    }

    let direct_require_candidates = dependencies_by_range
      .into_iter()
      .filter_map(|(expression_range, (dependency_id, count))| {
        (count == 1
          && require_header_ranges.iter().any(|header_range| {
            expression_range.start <= header_range.start && header_range.end <= expression_range.end
          }))
        .then_some(dependency_id)
      })
      .chain(self_rendering_require_dependencies);
    for dependency_id in direct_require_candidates {
      let dependency = module_graph.dependency_by_id(&dependency_id);
      // Keep optional requests on the normal module path so their failure and
      // module-cache behavior remains owned by the CommonJS runtime.
      if dependency
        .as_module_dependency()
        .is_some_and(|dependency| dependency.get_optional())
      {
        continue;
      }
      let Some(external_module) = module_graph
        .module_identifier_by_dependency_id(&dependency_id)
        .and_then(|module_id| module_graph.module_by_identifier(module_id))
        .and_then(|module| module.as_external_module())
      else {
        continue;
      };

      let request = external_module.get_request();
      if CommonJsExternalRequireKind::from_external_type(external_module.resolve_external_type())
        .is_some()
        && !request.has_rest()
        // Relative calls must stay in the external module's emitted directory.
        && !is_relative_external_request(request.primary())
      {
        direct_dependencies.insert(dependency_id);
      }
    }
  }

  let module_graph = build_module_graph_artifact.get_module_graph_mut();
  for dependency_id in &direct_dependencies {
    module_graph
      .connection_by_dependency_id_mut(dependency_id)
      .expect("direct CommonJS external should have a module graph connection")
      .force_inactive();
  }

  direct_dependencies
}

fn get_direct_external_require(
  dependency_id: &DependencyId,
  direct_dependencies: &DirectCommonJsExternalDependencies,
  context: &TemplateContext,
) -> Option<(String, CommonJsExternalRequireKind)> {
  if !direct_dependencies.borrow().contains(dependency_id) {
    return None;
  }

  let module_graph = context.compilation.get_module_graph();
  let external_module = module_graph
    .module_identifier_by_dependency_id(dependency_id)
    .and_then(|module_id| module_graph.module_by_identifier(module_id))
    .and_then(|module| module.as_external_module())?;

  Some((
    external_module.get_request().primary().to_string(),
    CommonJsExternalRequireKind::from_external_type(external_module.resolve_external_type())?,
  ))
}

#[derive(Debug)]
pub struct DirectCommonJsDependencyTemplate {
  pub direct_dependencies: DirectCommonJsExternalDependencies,
  pub template: Option<Arc<dyn DependencyTemplate>>,
}

impl DependencyTemplate for DirectCommonJsDependencyTemplate {
  fn render(
    &self,
    dependency: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    context: &mut TemplateContext,
  ) {
    let require = dependency
      .as_any()
      .downcast_ref::<CommonJsRequireDependency>();
    let full_require = dependency
      .as_any()
      .downcast_ref::<CommonJsFullRequireDependency>();
    let export_require = dependency
      .as_any()
      .downcast_ref::<CommonJsExportRequireDependency>();
    let id = require
      .map(Dependency::id)
      .or_else(|| full_require.map(Dependency::id))
      .or_else(|| export_require.map(Dependency::id))
      .expect("only CommonJS dependency templates are wrapped");
    let Some((request, kind)) = get_direct_external_require(id, &self.direct_dependencies, context)
    else {
      if let Some(template) = &self.template {
        template.render(dependency, source, context);
      }
      return;
    };

    if let Some(dep) = require {
      let range = dep.request_range();
      source.replace(range.start, range.end, json_stringify_str(&request), None);
      let header = (
        dep.range().expect("direct require has an expression range"),
        kind,
      );
      if let Some(headers) = context.data.get_mut::<DirectExternalRequireHeaders>() {
        headers.0.push(header);
      } else {
        context
          .data
          .insert(DirectExternalRequireHeaders(vec![header]));
      }
    } else if let Some(dep) = full_require {
      let mut expression = kind.render_expression(
        Some(&request),
        dep.names(),
        context.compilation,
        context.chunk_init_fragments(),
      );
      if dep.asi_safe() {
        expression = format!("({expression})");
      }
      let range = dep.range();
      source.replace(range.start, range.end, expression, None);
    } else if let Some(dep) = export_require {
      let expression = kind.render_expression(
        Some(&request),
        dep.get_ids(context.compilation.get_module_graph()),
        context.compilation,
        context.chunk_init_fragments(),
      );
      CommonJsExportRequireDependencyTemplate.render_with_require(
        dep,
        source,
        context,
        Some(expression),
      );
    }
  }
}

#[derive(Debug)]
pub struct DirectRequireHeaderDependencyTemplate {
  pub template: Option<Arc<dyn DependencyTemplate>>,
}

impl DependencyTemplate for DirectRequireHeaderDependencyTemplate {
  fn render(
    &self,
    dependency: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    context: &mut TemplateContext,
  ) {
    let dependency = dependency
      .as_any()
      .downcast_ref::<RequireHeaderDependency>()
      .expect(
        "DirectRequireHeaderDependencyTemplate should only be used for RequireHeaderDependency",
      );
    let header_range = dependency.range();
    let direct_require_kind =
      context
        .data
        .get::<DirectExternalRequireHeaders>()
        .and_then(|headers| {
          headers
            .0
            .iter()
            .find(|(expression_range, _)| {
              expression_range.start <= header_range.start
                && header_range.end <= expression_range.end
            })
            .map(|(_, kind)| *kind)
        });

    if let Some(kind) = direct_require_kind {
      let require = kind.render_callee(context.compilation, context.chunk_init_fragments());
      source.replace_static(header_range.start, header_range.end, require, None);
    } else if let Some(template) = &self.template {
      template.render(dependency, source, context);
    }
  }
}
