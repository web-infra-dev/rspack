use rspack_cacheable::{cacheable, with::AsCacheable};
use rspack_core::{
  ConnectionState, Dependency, DependencyCondition, DependencyConditionFn, DependencyId,
  EvaluatedInlinableValue, ExportsInfoArtifact, ModuleGraph, ModuleGraphCacheArtifact,
  ModuleGraphConnection, RuntimeSpec, SideEffectsStateArtifact, UsedName,
};

use super::{CommonJsRequireDependency, ESMImportSpecifierDependency, ImportDependency};
use crate::utils::eval::DependencyData;

#[cacheable]
#[derive(Debug, Clone)]
pub struct DependencyBranchGuard(#[cacheable(with=AsCacheable)] DependencyData);

impl DependencyBranchGuard {
  pub fn new(data: DependencyData) -> Self {
    Self(data)
  }

  pub fn into_inner(self) -> DependencyData {
    self.0
  }

  pub fn and(self, other: DependencyBranchGuard) -> Self {
    Self(self.0.and(other.0))
  }

  pub fn bind_dependency(&self, dep: &mut dyn Dependency) {
    if let Some(dep) = dep.downcast_mut::<ESMImportSpecifierDependency>() {
      dep.set_branch_guard(self.clone());
    } else if let Some(dep) = dep.downcast_mut::<ImportDependency>() {
      dep.set_branch_guard(self.clone());
    } else if let Some(dep) = dep.downcast_mut::<CommonJsRequireDependency>() {
      dep.set_branch_guard(self.clone());
    }
  }
}

pub fn compose_dependency_condition(
  base: Option<DependencyCondition>,
  branch_guard: Option<&DependencyBranchGuard>,
) -> Option<DependencyCondition> {
  let Some(branch_guard) = branch_guard else {
    return base;
  };

  Some(DependencyCondition::new(BranchGuardDependencyCondition {
    base,
    branch_guard: branch_guard.clone(),
  }))
}

struct BranchGuardDependencyCondition {
  base: Option<DependencyCondition>,
  branch_guard: DependencyBranchGuard,
}

impl DependencyConditionFn for BranchGuardDependencyCondition {
  fn get_connection_state(
    &self,
    conn: &ModuleGraphConnection,
    runtime: Option<&RuntimeSpec>,
    module_graph: &ModuleGraph,
    module_graph_cache: &ModuleGraphCacheArtifact,
    side_effects_state_artifact: &SideEffectsStateArtifact,
    exports_info_artifact: &ExportsInfoArtifact,
  ) -> ConnectionState {
    if matches!(
      resolve_branch_guard(
        &self.branch_guard,
        runtime,
        module_graph,
        exports_info_artifact
      ),
      Some(false)
    ) {
      return ConnectionState::Active(false);
    }

    if let Some(condition) = &self.base {
      condition.get_connection_state(
        conn,
        runtime,
        module_graph,
        module_graph_cache,
        side_effects_state_artifact,
        exports_info_artifact,
      )
    } else {
      ConnectionState::Active(true)
    }
  }
}

fn resolve_branch_guard(
  condition: &DependencyBranchGuard,
  runtime: Option<&RuntimeSpec>,
  module_graph: &ModuleGraph,
  exports_info_artifact: &ExportsInfoArtifact,
) -> Option<bool> {
  resolve_dependency_data(&condition.0, runtime, module_graph, exports_info_artifact)
}

fn resolve_dependency_data(
  data: &DependencyData,
  runtime: Option<&RuntimeSpec>,
  module_graph: &ModuleGraph,
  exports_info_artifact: &ExportsInfoArtifact,
) -> Option<bool> {
  match data {
    DependencyData::Dependency(dependency_id) => resolve_esm_imported_boolean_guard(
      dependency_id,
      runtime,
      module_graph,
      exports_info_artifact,
    ),
    DependencyData::Or(left, right) => {
      let left = resolve_dependency_data(left, runtime, module_graph, exports_info_artifact);
      if matches!(left, Some(true)) {
        return Some(true);
      }
      let right = resolve_dependency_data(right, runtime, module_graph, exports_info_artifact);
      match (left, right) {
        (Some(true), _) | (_, Some(true)) => Some(true),
        (Some(false), Some(false)) => Some(false),
        _ => None,
      }
    }
    DependencyData::And(left, right) => {
      let left = resolve_dependency_data(left, runtime, module_graph, exports_info_artifact);
      if matches!(left, Some(false)) {
        return Some(false);
      }
      let right = resolve_dependency_data(right, runtime, module_graph, exports_info_artifact);
      match (left, right) {
        (Some(true), Some(true)) => Some(true),
        (Some(false), _) | (_, Some(false)) => Some(false),
        _ => None,
      }
    }
    DependencyData::Not(data) => {
      resolve_dependency_data(data, runtime, module_graph, exports_info_artifact)
        .map(|value| !value)
    }
  }
}

fn resolve_esm_imported_boolean_guard(
  dependency_id: &DependencyId,
  runtime: Option<&RuntimeSpec>,
  module_graph: &ModuleGraph,
  exports_info_artifact: &ExportsInfoArtifact,
) -> Option<bool> {
  let dependency = module_graph
    .dependency_by_id(dependency_id)
    .downcast_ref::<ESMImportSpecifierDependency>()?;
  let ids = dependency.get_ids(module_graph);
  if ids.is_empty() {
    return None;
  }

  let module_identifier = module_graph.module_identifier_by_dependency_id(dependency_id)?;
  let exports_info = exports_info_artifact.get_exports_info_data(module_identifier);
  let used_name = exports_info.get_used_name(exports_info_artifact, runtime, ids)?;
  let UsedName::Inlined(inlined) = used_name else {
    return None;
  };

  match inlined.inlined_value() {
    EvaluatedInlinableValue::Boolean(value) => Some(*value),
    _ => None,
  }
}
