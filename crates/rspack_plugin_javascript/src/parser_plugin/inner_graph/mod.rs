use std::rc::Rc;

use rspack_core::{
  Compilation, ExportsInfoArtifact, GetTargetResult, ModuleGraph, ModuleGraphConnection,
  ModuleIdentifier, PrefetchExportsInfoMode, RuntimeCondition, RuntimeSpec, UsageState,
  UsedByExports, UsedByExportsCondition, filter_runtime, get_target,
};

pub mod plugin;

pub(crate) fn has_impure_deferred_pure_checks(
  module_graph: &ModuleGraph,
  exports_info_artifact: &ExportsInfoArtifact,
  used_by_exports: &UsedByExports,
) -> bool {
  used_by_exports
    .deferred_pure_checks
    .iter()
    .any(|deferred_check| {
      let Some(ref_module) =
        module_graph.module_identifier_by_dependency_id(&deferred_check.dep_id)
      else {
        return true;
      };

      let target_exports_info = exports_info_artifact
        .get_prefetched_exports_info(ref_module, PrefetchExportsInfoMode::Default);
      let target_export_info =
        target_exports_info.get_export_info_without_mut_module_graph(&deferred_check.atom);

      let (ref_module_id, atom) = if let Some(GetTargetResult::Target(target)) = get_target(
        &target_export_info,
        module_graph,
        exports_info_artifact,
        Rc::new(|_| true),
        &mut Default::default(),
      ) && let Some(export) = &target.export
        && let Some(atom) = export.first()
      {
        (target.module, atom.clone())
      } else {
        (*ref_module, deferred_check.atom.clone())
      };

      let Some(ref_module) = module_graph.module_by_identifier(&ref_module_id) else {
        return true;
      };
      let Some(side_effects_free) = &ref_module.build_info().side_effects_free else {
        return true;
      };

      !side_effects_free.contains(&atom)
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

  match &used_by_exports.condition {
    UsedByExportsCondition::Bool(used) => RuntimeCondition::Boolean(*used),
    UsedByExportsCondition::Set(used_by_exports) => {
      let exports_info = compilation
        .exports_info_artifact
        .get_prefetched_exports_info(module_identifier, PrefetchExportsInfoMode::Default);
      filter_runtime(runtime, |cur_runtime| {
        used_by_exports.iter().any(|name| {
          exports_info.get_used(std::slice::from_ref(name), cur_runtime) != UsageState::Unused
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
  let used_by_exports = match &used_by_exports.condition {
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
