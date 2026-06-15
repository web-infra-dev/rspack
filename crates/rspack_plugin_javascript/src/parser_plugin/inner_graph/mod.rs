use rspack_core::{
  Compilation, DependencyId, ExportsInfoArtifact, GetTargetResult, ModuleGraph,
  ModuleGraphConnection, ModuleIdentifier, ResolvedExportInfoTarget, RuntimeCondition, RuntimeSpec,
  UsageState, UsedByExports, UsedByExportsCondition, filter_runtime, get_target,
};
use swc_atoms::Atom;

pub mod plugin;
pub mod state;

fn module_has_side_effects_free_export(
  module_graph: &ModuleGraph,
  module_identifier: &ModuleIdentifier,
  atom: &Atom,
) -> Option<bool> {
  let module = module_graph.module_by_identifier(module_identifier)?;
  let side_effects_free = module.build_info().side_effects_free.as_ref()?;
  Some(side_effects_free.contains(atom))
}

pub(crate) fn deferred_pure_check_is_impure(
  module_graph: &ModuleGraph,
  exports_info_artifact: &ExportsInfoArtifact,
  dep_id: &DependencyId,
  atom: &Atom,
) -> bool {
  let Some(ref_module) = module_graph.module_identifier_by_dependency_id(dep_id) else {
    return true;
  };

  let target_exports_info = exports_info_artifact.get_exports_info_data(ref_module);
  let target_export_info = target_exports_info.get_export_info_without_mut_module_graph(atom);
  let resolve_filter = |_: &ResolvedExportInfoTarget| true;

  let (ref_module_id, resolved_atom) = if let Some(GetTargetResult::Target(target)) = get_target(
    &target_export_info,
    module_graph,
    exports_info_artifact,
    &resolve_filter,
    &mut Default::default(),
  ) {
    let atom = if target.module == *ref_module {
      Some(atom.clone())
    } else {
      target
        .export
        .as_ref()
        .and_then(|export| export.first().cloned())
    };
    (target.module, atom)
  } else {
    (*ref_module, Some(atom.clone()))
  };

  if let Some(resolved_atom) = resolved_atom.as_ref()
    && let Some(side_effects_free) =
      module_has_side_effects_free_export(module_graph, &ref_module_id, resolved_atom)
  {
    return !side_effects_free;
  }

  if let Some(resolved_module) = module_graph.get_resolved_module(dep_id)
    && resolved_module != &ref_module_id
    && let Some(side_effects_free) =
      module_has_side_effects_free_export(module_graph, resolved_module, atom)
  {
    return !side_effects_free;
  }

  true
}

pub(crate) fn has_impure_deferred_pure_checks(
  module_graph: &ModuleGraph,
  exports_info_artifact: &ExportsInfoArtifact,
  used_by_exports: &UsedByExports,
) -> bool {
  used_by_exports
    .deferred_pure_checks()
    .iter()
    .any(|deferred_check| {
      deferred_pure_check_is_impure(
        module_graph,
        exports_info_artifact,
        &deferred_check.dep_id,
        &deferred_check.atom,
      )
    })
}

pub(crate) fn runtime_condition_used_by_exports(
  compilation: &Compilation,
  module_identifier: &ModuleIdentifier,
  runtime: Option<&RuntimeSpec>,
  used_by_exports: Option<&UsedByExports>,
) -> RuntimeCondition {
  let Some(used_by_exports) = used_by_exports else {
    return RuntimeCondition::Boolean(true);
  };

  if has_impure_deferred_pure_checks(
    compilation.get_module_graph(),
    &compilation.exports_info_artifact,
    used_by_exports,
  ) {
    return RuntimeCondition::Boolean(true);
  }

  match used_by_exports.condition() {
    UsedByExportsCondition::Bool(used) => RuntimeCondition::Boolean(*used),
    UsedByExportsCondition::Set(used_by_exports) => {
      let exports_info = compilation
        .exports_info_artifact
        .get_exports_info_data(module_identifier);
      filter_runtime(runtime, |cur_runtime| {
        used_by_exports.iter().any(|name| {
          exports_info.get_used(
            &compilation.exports_info_artifact,
            std::slice::from_ref(name),
            cur_runtime,
          ) != UsageState::Unused
        })
      })
    }
  }
}

pub fn connection_active_used_by_exports(
  connection: &ModuleGraphConnection,
  runtime: Option<&RuntimeSpec>,
  mg: &ModuleGraph,
  exports_info_artifact: &ExportsInfoArtifact,
  used_by_exports: Option<&UsedByExports>,
) -> bool {
  let Some(used_by_exports) = used_by_exports.as_ref() else {
    return true;
  };
  if has_impure_deferred_pure_checks(mg, exports_info_artifact, used_by_exports) {
    return true;
  }
  let used_by_exports = match used_by_exports.condition() {
    UsedByExportsCondition::Set(used_by_exports) => used_by_exports,
    UsedByExportsCondition::Bool(used) => return *used,
  };
  let module_identifier = mg
    .get_parent_module(&connection.dependency_id)
    .expect("should have parent module");
  let exports_info = exports_info_artifact.get_exports_info_data(module_identifier);
  used_by_exports.iter().any(|name| {
    exports_info
      .named_exports(name)
      .unwrap_or_else(|| exports_info.other_exports_info())
      .get_used(runtime)
      != UsageState::Unused
  })
}
