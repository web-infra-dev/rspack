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
  ids: &[Atom],
) -> Option<bool> {
  let module = module_graph.module_by_identifier(module_identifier)?;
  let side_effects_free = module.build_info().side_effects_free.as_ref()?;
  Some(
    ids
      .last()
      .is_some_and(|atom| side_effects_free.contains(atom)),
  )
}

fn resolved_target_is_side_effects_free(
  module_graph: &ModuleGraph,
  exports_info_artifact: &ExportsInfoArtifact,
  module_identifier: &ModuleIdentifier,
  ids: &[Atom],
) -> Option<bool> {
  let exports_info = exports_info_artifact.get_exports_info_data(module_identifier);
  let export_info = exports_info.get_read_only_export_info_recursive(exports_info_artifact, ids)?;
  let resolve_filter = |_: &ResolvedExportInfoTarget| true;

  if let Some(GetTargetResult::Target(target)) = get_target(
    export_info,
    module_graph,
    exports_info_artifact,
    &resolve_filter,
    &mut Default::default(),
  ) {
    let resolved_ids = target.export.as_deref().unwrap_or(ids);
    module_has_side_effects_free_export(module_graph, &target.module, resolved_ids)
  } else {
    module_has_side_effects_free_export(module_graph, module_identifier, ids)
  }
}

pub fn deferred_pure_check_is_impure(
  module_graph: &ModuleGraph,
  exports_info_artifact: &ExportsInfoArtifact,
  dep_id: &DependencyId,
  ids: &[Atom],
) -> bool {
  if ids.is_empty() {
    return true;
  }
  let Some(ref_module) = module_graph.module_identifier_by_dependency_id(dep_id) else {
    return true;
  };

  let target_exports_info = exports_info_artifact.get_exports_info_data(ref_module);
  let Some(target_export_info) =
    target_exports_info.get_read_only_export_info_recursive(exports_info_artifact, ids)
  else {
    return true;
  };
  let resolve_filter = |_: &ResolvedExportInfoTarget| true;

  let (ref_module_id, resolved_ids) = if let Some(GetTargetResult::Target(target)) = get_target(
    target_export_info,
    module_graph,
    exports_info_artifact,
    &resolve_filter,
    &mut Default::default(),
  ) {
    let ids = if target.module == *ref_module {
      Some(ids.to_vec())
    } else {
      target.export
    };
    (target.module, ids)
  } else {
    (*ref_module, Some(ids.to_vec()))
  };

  if let Some(resolved_ids) = resolved_ids.as_deref()
    && let Some(side_effects_free) =
      module_has_side_effects_free_export(module_graph, &ref_module_id, resolved_ids)
  {
    return !side_effects_free;
  }

  // Namespace reexports such as `export * as pure from "./source"` expose a nested
  // exports info object whose target is the source module namespace. Resolve that
  // first segment, then continue checking the remaining property path in the source
  // module. This mirrors webpack's recursive ExportInfo target lookup for `pure.fn`.
  if ids.len() > 1 {
    let first_export_info = target_exports_info.get_read_only_export_info(&ids[0]);
    if let Some(GetTargetResult::Target(target)) = get_target(
      first_export_info,
      module_graph,
      exports_info_artifact,
      &resolve_filter,
      &mut Default::default(),
    ) {
      let mut remaining_ids = target.export.unwrap_or_default();
      remaining_ids.extend_from_slice(&ids[1..]);
      if let Some(side_effects_free) = resolved_target_is_side_effects_free(
        module_graph,
        exports_info_artifact,
        &target.module,
        &remaining_ids,
      ) {
        return !side_effects_free;
      }
    }
  }

  if let Some(resolved_module) = module_graph.get_resolved_module(dep_id)
    && resolved_module != &ref_module_id
    && let Some(side_effects_free) =
      module_has_side_effects_free_export(module_graph, resolved_module, ids)
  {
    return !side_effects_free;
  }

  true
}

pub fn has_impure_deferred_pure_checks(
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
        &deferred_check.ids,
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
