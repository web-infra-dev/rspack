use std::{collections::hash_map::Entry, sync::Arc};

use atomic_refcell::AtomicRefCell;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  BuildModuleGraphArtifact, CodeGenerationDataItem, CommonJsExternalRequireKind,
  ConnectionActiveState, Dependency, DependencyCodeGeneration, DependencyId, DependencyRange,
  DependencyTemplate, TemplateContext, TemplateReplaceSource, UsedName, property_access,
};
use rspack_plugin_javascript::dependency::{
  CommonJsExportRequireDependency, CommonJsFullRequireDependency, CommonJsRequireDependency,
  RequireHeaderDependency,
};
use rspack_util::{
  fx_hash::{FxHashMap, FxHashSet},
  json_stringify_str,
};

pub type DirectCommonJsExternalDependencies = Arc<AtomicRefCell<Arc<FxHashSet<DependencyId>>>>;
pub type DirectCommonJsExternalConnectionStates =
  AtomicRefCell<FxHashMap<DependencyId, ConnectionActiveState>>;

#[cacheable]
#[derive(Debug, Clone)]
struct DirectExternalRequireHeaders(Vec<(DependencyRange, CommonJsExternalRequireKind)>);

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

fn can_render_direct_request(request: &str) -> bool {
  // Both ambient `require` and `createRequire(import.meta.url)` resolve
  // relative requests from the emitted asset containing the call. Keep those
  // requests on the external-module path because direct rendering can move the
  // call into an issuer emitted in another directory.
  !is_relative_external_request(request)
}

pub fn cutout_commonjs_externals(
  build_module_graph_artifact: &mut BuildModuleGraphArtifact,
  connection_states: &mut FxHashMap<DependencyId, ConnectionActiveState>,
) -> FxHashSet<DependencyId> {
  let module_graph = build_module_graph_artifact.get_module_graph();
  let mut direct_dependencies = FxHashSet::default();

  for (_, module) in module_graph.modules() {
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

    for dependency_id in module.get_dependencies() {
      let dependency = module_graph.dependency_by_id(dependency_id);
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
      .into_values()
      .filter_map(|(dependency_id, count)| {
        if count != 1 {
          return None;
        }
        let expression_range = module_graph
          .dependency_by_id(&dependency_id)
          .range()
          .expect("grouped CommonJS require should have an expression range");
        require_header_ranges
          .iter()
          .any(|header_range| {
            expression_range.start <= header_range.start && header_range.end <= expression_range.end
          })
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
        && can_render_direct_request(request.primary())
      {
        direct_dependencies.insert(dependency_id);
      }
    }
  }

  let module_graph = build_module_graph_artifact.get_module_graph_mut();
  // A reused module graph can make a previously direct dependency ineligible.
  // Restore the connection state captured before this plugin cut it out.
  connection_states.retain(|dependency_id, state| {
    if direct_dependencies.contains(dependency_id) {
      return module_graph
        .connection_by_dependency_id(dependency_id)
        .is_some();
    }
    if let Some(connection) = module_graph.connection_by_dependency_id_mut(dependency_id) {
      connection.restore_active_state(*state);
    }
    false
  });
  for dependency_id in &direct_dependencies {
    let connection = module_graph
      .connection_by_dependency_id_mut(dependency_id)
      .expect("direct CommonJS external should have a module graph connection");
    match connection_states.entry(*dependency_id) {
      Entry::Vacant(entry) => {
        entry.insert(connection.force_inactive_with_state());
      }
      Entry::Occupied(_) => connection.force_inactive(),
    }
  }

  direct_dependencies
}

fn get_direct_external_require(
  dependency_id: &DependencyId,
  expression_range: DependencyRange,
  direct_dependencies: &DirectCommonJsExternalDependencies,
  context: &TemplateContext,
) -> Option<(String, DependencyRange, CommonJsExternalRequireKind)> {
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
    expression_range,
    CommonJsExternalRequireKind::from_external_type(external_module.resolve_external_type())?,
  ))
}

fn render_direct_require(
  request: &str,
  properties: &[rspack_util::atom::Atom],
  kind: CommonJsExternalRequireKind,
  context: &mut TemplateContext,
) -> String {
  let compilation = context.compilation;
  kind.render_expression(
    Some(request),
    properties,
    compilation,
    context.chunk_init_fragments(),
  )
}

#[derive(Debug)]
pub struct DirectCommonJsRequireDependencyTemplate {
  pub direct_dependencies: DirectCommonJsExternalDependencies,
  pub template: Option<Arc<dyn DependencyTemplate>>,
}

impl DependencyTemplate for DirectCommonJsRequireDependencyTemplate {
  fn render(
    &self,
    dependency: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    context: &mut TemplateContext,
  ) {
    let dependency = dependency
      .as_any()
      .downcast_ref::<CommonJsRequireDependency>()
      .expect(
        "DirectCommonJsRequireDependencyTemplate should only be used for CommonJsRequireDependency",
      );

    let Some(expression_range) = dependency.range() else {
      if let Some(template) = &self.template {
        template.render(dependency, source, context);
      }
      return;
    };
    let Some((request, expression_range, kind)) = get_direct_external_require(
      dependency.id(),
      expression_range,
      &self.direct_dependencies,
      context,
    ) else {
      if let Some(template) = &self.template {
        template.render(dependency, source, context);
      }
      return;
    };

    let request_range = dependency.request_range();
    source.replace(
      request_range.start,
      request_range.end,
      json_stringify_str(&request),
      None,
    );

    if context.data.get::<DirectExternalRequireHeaders>().is_none() {
      context
        .data
        .insert(DirectExternalRequireHeaders(Vec::new()));
    }
    context
      .data
      .get_mut::<DirectExternalRequireHeaders>()
      .expect("direct external require headers should be initialized")
      .0
      .push((expression_range, kind));
  }
}

#[derive(Debug)]
pub struct DirectCommonJsExportRequireDependencyTemplate {
  pub direct_dependencies: DirectCommonJsExternalDependencies,
  pub template: Option<Arc<dyn DependencyTemplate>>,
}

impl DependencyTemplate for DirectCommonJsExportRequireDependencyTemplate {
  fn render(
    &self,
    dependency: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    context: &mut TemplateContext,
  ) {
    let dependency = dependency
      .as_any()
      .downcast_ref::<CommonJsExportRequireDependency>()
      .expect(
        "DirectCommonJsExportRequireDependencyTemplate should only be used for CommonJsExportRequireDependency",
      );
    let range = dependency.range();
    let Some((request, _, kind)) =
      get_direct_external_require(dependency.id(), range, &self.direct_dependencies, context)
    else {
      if let Some(template) = &self.template {
        template.render(dependency, source, context);
      }
      return;
    };

    let module_graph = context.compilation.get_module_graph();
    let module = module_graph
      .module_by_identifier(&context.module.identifier())
      .expect("CommonJS export require should have a module graph module");
    let base = dependency.base();
    let base = if base.is_exports() {
      context
        .runtime_template
        .render_exports_argument(module.get_exports_argument())
    } else if base.is_module_exports() {
      format!(
        "{}.exports",
        context
          .runtime_template
          .render_module_argument(module.get_module_argument())
      )
    } else if base.is_this() {
      context.runtime_template.render_this_exports()
    } else {
      unreachable!("CommonJS export require should use an expression base")
    };
    let used = context
      .compilation
      .exports_info_artifact
      .get_exports_info_data(&module.identifier())
      .get_used_name(
        &context.compilation.exports_info_artifact,
        context.runtime,
        dependency.names(),
      );
    let require_expression =
      render_direct_require(&request, dependency.get_ids(module_graph), kind, context);

    let expression = match used {
      Some(UsedName::Normal(used)) => {
        format!("{base}{} = {require_expression}", property_access(used, 0))
      }
      Some(UsedName::Inlined(_)) => {
        format!("/* inlined reexport */ {require_expression}")
      }
      None => format!("/* unused reexport */ {require_expression}"),
    };
    source.replace(range.start, range.end, expression, None);
  }
}

#[derive(Debug)]
pub struct DirectCommonJsFullRequireDependencyTemplate {
  pub direct_dependencies: DirectCommonJsExternalDependencies,
  pub template: Option<Arc<dyn DependencyTemplate>>,
}

impl DependencyTemplate for DirectCommonJsFullRequireDependencyTemplate {
  fn render(
    &self,
    dependency: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    context: &mut TemplateContext,
  ) {
    let dependency = dependency
      .as_any()
      .downcast_ref::<CommonJsFullRequireDependency>()
      .expect(
        "DirectCommonJsFullRequireDependencyTemplate should only be used for CommonJsFullRequireDependency",
      );
    let range = dependency.range();
    let Some((request, _, kind)) =
      get_direct_external_require(dependency.id(), range, &self.direct_dependencies, context)
    else {
      if let Some(template) = &self.template {
        template.render(dependency, source, context);
      }
      return;
    };

    let mut expression = render_direct_require(&request, dependency.names(), kind, context);
    if dependency.asi_safe() {
      expression = format!("({expression})");
    }
    source.replace(range.start, range.end, expression, None);
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

    match direct_require_kind {
      Some(kind) => {
        let compilation = context.compilation;
        let require = kind
          .render_callee(compilation, context.chunk_init_fragments())
          .to_string();
        source.replace(header_range.start, header_range.end, require, None);
      }
      None => {
        if let Some(template) = &self.template {
          template.render(dependency, source, context);
        }
      }
    }
  }
}
