use std::sync::Arc;

use atomic_refcell::AtomicRefCell;
use rspack_core::{
  BuildModuleGraphArtifact, Compilation, Dependency, DependencyCodeGeneration, DependencyId,
  DependencyTemplate, ExportsInfoArtifact, ExternalModule, ImportPhase, ModuleGraphConnection,
  TemplateContext, TemplateReplaceSource,
};
use rspack_plugin_javascript::dependency::{
  ESMExportImportedSpecifierDependency, ESMExportImportedSpecifierDependencyTemplate,
  ESMImportSideEffectDependency, ESMImportSpecifierDependency,
  ESMImportSpecifierDependencyTemplate, esm_import_external,
};
use rspack_util::fx_hash::FxHashMap;

use super::{ExternalBindingBailout, commonjs_external::is_relative_external_request};

pub type DirectEsmExternalDependencies =
  Arc<AtomicRefCell<FxHashMap<DependencyId, ModuleGraphConnection>>>;

pub fn requires_issuer_identity(external: &ExternalModule) -> bool {
  let request = external.get_request();
  // Relative imports must remain in the issuer's emitted directory. Projected
  // namespaces are expressions, not bindings that can be exported by a shared chunk.
  request.has_rest() || is_relative_external_request(request.primary())
}

pub fn can_render_esm_external(dependency: &dyn Dependency, external: &ExternalModule) -> bool {
  external.resolve_external_type() == "module"
    && !dependency
      .as_module_dependency()
      .is_some_and(|dependency| dependency.weak())
    && external.get_import_phase() == ImportPhase::Evaluation
    && !requires_issuer_identity(external)
    && (dependency.as_any().is::<ESMImportSideEffectDependency>()
      || dependency.as_any().is::<ESMImportSpecifierDependency>()
      || dependency
        .as_any()
        .is::<ESMExportImportedSpecifierDependency>())
}

pub fn cutout_esm_externals(
  compilation: &Compilation,
  artifact: &mut BuildModuleGraphArtifact,
  exports_info: &ExportsInfoArtifact,
  direct: &DirectEsmExternalDependencies,
) {
  let mg = artifact.get_module_graph();
  let mut previous = std::mem::take(&mut *direct.borrow_mut());
  let mut connections = FxHashMap::default();
  for (_, module) in mg.modules() {
    if ExternalBindingBailout::is_present(module.as_ref()) {
      continue;
    }
    for dependency in module.get_dependencies() {
      let dependency = dependency.as_ref();
      let id = dependency.id();
      let Some(connection) = mg.connection_by_dependency_id(id) else {
        continue;
      };
      let Some(external) = mg
        .module_by_identifier(connection.module_identifier())
        .and_then(|module| module.as_external_module())
      else {
        continue;
      };
      // Checked/partial reexports still need their original template semantics.
      // Keep their edge, instead of changing the issuer's scope-hoisting decision.
      if let Some(dependency) = dependency
        .as_any()
        .downcast_ref::<ESMExportImportedSpecifierDependency>()
        && !ESMExportImportedSpecifierDependencyTemplate::supports_direct_scope_export(
          &dependency.get_mode(
            mg,
            None,
            &compilation.module_graph_cache_artifact,
            exports_info,
          ),
        )
      {
        continue;
      }
      if can_render_esm_external(dependency, external)
        && (previous.contains_key(id)
          || connection.is_active(
            mg,
            None,
            &compilation.module_graph_cache_artifact,
            &artifact.side_effects_state_artifact,
            exports_info,
          ))
      {
        // OptimizeDependencies may run more than once. Preserve the semantic
        // condition, while only disabling placement in the chunk graph.
        connections.insert(
          *id,
          previous.remove(id).unwrap_or_else(|| connection.clone()),
        );
      }
    }
  }
  let mg = artifact.get_module_graph_mut();
  for id in connections.keys() {
    mg.connection_by_dependency_id_mut(id)
      .expect("external connection should exist")
      .force_inactive();
  }
  *direct.borrow_mut() = connections;
}

#[derive(Debug)]
pub struct DirectEsmExternalDependencyTemplate {
  pub direct: DirectEsmExternalDependencies,
  pub template: Arc<dyn DependencyTemplate>,
}

impl DependencyTemplate for DirectEsmExternalDependencyTemplate {
  fn render(
    &self,
    dependency: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    context: &mut TemplateContext,
  ) {
    let side_effect = dependency
      .as_any()
      .downcast_ref::<ESMImportSideEffectDependency>();
    let specifier = dependency
      .as_any()
      .downcast_ref::<ESMImportSpecifierDependency>();
    let reexport = dependency
      .as_any()
      .downcast_ref::<ESMExportImportedSpecifierDependency>();
    let id = side_effect
      .map(Dependency::id)
      .or_else(|| specifier.map(Dependency::id))
      .or_else(|| reexport.map(Dependency::id))
      .expect("only ESM dependency templates are wrapped");
    let direct = self.direct.borrow();
    let Some(connection) = direct.get(id) else {
      self.template.render(dependency, source, context);
      return;
    };
    let compilation = context.compilation;
    let mg = compilation.get_module_graph();
    if !connection.is_target_active(
      mg,
      context.runtime,
      &compilation.module_graph_cache_artifact,
      &compilation
        .build_module_graph_artifact
        .side_effects_state_artifact,
      &compilation.exports_info_artifact,
    ) {
      return;
    }
    let external = mg
      .module_by_identifier(connection.module_identifier())
      .and_then(|module| module.as_external_module())
      .expect("direct dependency should target an external");
    if let Some(dep) = side_effect {
      if let Some(scope) = context.concatenation_scope.as_mut() {
        external.register_module_side_effect(scope);
      } else {
        esm_import_external(external, dep, context);
      }
    } else if let Some(dep) = specifier {
      ESMImportSpecifierDependencyTemplate.render_with_external(
        dep,
        source,
        context,
        Some((external, connection)),
      );
    } else if let Some(dep) = reexport {
      ESMExportImportedSpecifierDependencyTemplate
        .render_external(dep, external, connection, context);
    }
  }
}
