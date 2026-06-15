use rspack_cacheable::{cacheable, with::AsCacheable};
use rspack_core::{
  ConnectionState, Dependency, DependencyCondition, DependencyConditionFn, DependencyId,
  EvaluatedInlinableValue, ExportProvided, ExportsInfoArtifact, ExportsType, ModuleGraph,
  ModuleGraphCacheArtifact, ModuleGraphConnection, RuntimeSpec, SideEffectsStateArtifact, UsedName,
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

pub fn is_dependency_export_presence_guarded(
  guard: &DependencyBranchGuard,
  dependency: &ESMImportSpecifierDependency,
  module_graph: &ModuleGraph,
) -> bool {
  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  enum KnownValue {
    Truthy,
    Falsy,
  }

  impl KnownValue {
    fn negate(self) -> Self {
      match self {
        Self::Truthy => Self::Falsy,
        Self::Falsy => Self::Truthy,
      }
    }
  }

  fn dependency_guards_export_presence(
    guard_dep: &DependencyId,
    dependency: &ESMImportSpecifierDependency,
    module_graph: &ModuleGraph,
  ) -> bool {
    let Some(guard_dep) = module_graph
      .dependency_by_id(guard_dep)
      .downcast_ref::<ESMImportSpecifierDependency>()
    else {
      return false;
    };
    if !guard_dep.evaluated_in_operator {
      return false;
    }
    if module_graph.module_identifier_by_dependency_id(guard_dep.id())
      != module_graph.module_identifier_by_dependency_id(dependency.id())
    {
      return false;
    }
    let guard_ids = guard_dep.get_ids(module_graph);
    if guard_ids.is_empty() {
      return false;
    }
    guard_ids == dependency.get_ids(module_graph)
  }

  fn implies_export_presence(
    data: &DependencyData,
    dependency: &ESMImportSpecifierDependency,
    module_graph: &ModuleGraph,
    known: KnownValue,
  ) -> bool {
    match data {
      DependencyData::Dependency(guard_dep) => {
        known == KnownValue::Truthy
          && dependency_guards_export_presence(guard_dep, dependency, module_graph)
      }
      DependencyData::And(left, right) => match known {
        // If `A && B` is truthy, both sides are truthy; either side can prove the export exists.
        KnownValue::Truthy => {
          implies_export_presence(left, dependency, module_graph, KnownValue::Truthy)
            || implies_export_presence(right, dependency, module_graph, KnownValue::Truthy)
        }
        // If `A && B` is falsy, at least one side is falsy; both falsy cases must prove it.
        KnownValue::Falsy => {
          implies_export_presence(left, dependency, module_graph, KnownValue::Falsy)
            && implies_export_presence(right, dependency, module_graph, KnownValue::Falsy)
        }
      },
      DependencyData::Or(left, right) => match known {
        // If `A || B` is truthy, only one side may be truthy; both truthy cases must prove it.
        KnownValue::Truthy => {
          implies_export_presence(left, dependency, module_graph, KnownValue::Truthy)
            && implies_export_presence(right, dependency, module_graph, KnownValue::Truthy)
        }
        // If `A || B` is falsy, both sides are falsy; either side can prove the export exists.
        KnownValue::Falsy => {
          implies_export_presence(left, dependency, module_graph, KnownValue::Falsy)
            || implies_export_presence(right, dependency, module_graph, KnownValue::Falsy)
        }
      },
      // If `!A` is known, reason about `A` with the opposite known value.
      DependencyData::Not(data) => {
        implies_export_presence(data, dependency, module_graph, known.negate())
      }
    }
  }

  implies_export_presence(&guard.0, dependency, module_graph, KnownValue::Truthy)
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
        module_graph_cache,
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
  module_graph_cache: &ModuleGraphCacheArtifact,
  exports_info_artifact: &ExportsInfoArtifact,
) -> Option<bool> {
  resolve_dependency_data(
    &condition.0,
    runtime,
    module_graph,
    module_graph_cache,
    exports_info_artifact,
  )
}

fn resolve_dependency_data(
  data: &DependencyData,
  runtime: Option<&RuntimeSpec>,
  module_graph: &ModuleGraph,
  module_graph_cache: &ModuleGraphCacheArtifact,
  exports_info_artifact: &ExportsInfoArtifact,
) -> Option<bool> {
  match data {
    DependencyData::Dependency(dependency_id) => resolve_esm_imported_boolean_guard(
      dependency_id,
      runtime,
      module_graph,
      module_graph_cache,
      exports_info_artifact,
    ),
    DependencyData::Or(left, right) => {
      let left = resolve_dependency_data(
        left,
        runtime,
        module_graph,
        module_graph_cache,
        exports_info_artifact,
      );
      if matches!(left, Some(true)) {
        return Some(true);
      }
      let right = resolve_dependency_data(
        right,
        runtime,
        module_graph,
        module_graph_cache,
        exports_info_artifact,
      );
      match (left, right) {
        (Some(true), _) | (_, Some(true)) => Some(true),
        (Some(false), Some(false)) => Some(false),
        _ => None,
      }
    }
    DependencyData::And(left, right) => {
      let left = resolve_dependency_data(
        left,
        runtime,
        module_graph,
        module_graph_cache,
        exports_info_artifact,
      );
      if matches!(left, Some(false)) {
        return Some(false);
      }
      let right = resolve_dependency_data(
        right,
        runtime,
        module_graph,
        module_graph_cache,
        exports_info_artifact,
      );
      match (left, right) {
        (Some(true), Some(true)) => Some(true),
        (Some(false), _) | (_, Some(false)) => Some(false),
        _ => None,
      }
    }
    DependencyData::Not(data) => resolve_dependency_data(
      data,
      runtime,
      module_graph,
      module_graph_cache,
      exports_info_artifact,
    )
    .map(|value| !value),
  }
}

fn resolve_esm_imported_boolean_guard(
  dependency_id: &DependencyId,
  runtime: Option<&RuntimeSpec>,
  module_graph: &ModuleGraph,
  module_graph_cache: &ModuleGraphCacheArtifact,
  exports_info_artifact: &ExportsInfoArtifact,
) -> Option<bool> {
  let dependency = module_graph
    .dependency_by_id(dependency_id)
    .downcast_ref::<ESMImportSpecifierDependency>()?;
  if dependency.evaluated_in_operator {
    return resolve_esm_imported_in_operator_guard(
      dependency,
      dependency_id,
      module_graph,
      module_graph_cache,
      exports_info_artifact,
    );
  }

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

fn resolve_esm_imported_in_operator_guard(
  dependency: &ESMImportSpecifierDependency,
  dependency_id: &DependencyId,
  module_graph: &ModuleGraph,
  module_graph_cache: &ModuleGraphCacheArtifact,
  exports_info_artifact: &ExportsInfoArtifact,
) -> Option<bool> {
  let ids = dependency.get_ids(module_graph);
  let first = ids.first()?;
  let module = module_graph.get_module_by_dependency_id(dependency_id)?;
  let parent_module_identifier = module_graph.get_parent_module(dependency_id)?;
  let parent_module = module_graph.module_by_identifier(parent_module_identifier)?;
  let exports_info = exports_info_artifact.get_exports_info_data(&module.identifier());
  let exports_type = module.get_exports_type(
    module_graph,
    module_graph_cache,
    exports_info_artifact,
    parent_module.build_meta().strict_esm_module,
  );
  let provided = match exports_type {
    ExportsType::DefaultWithNamed => {
      if first == "default" {
        if ids.len() == 1 {
          Some(ExportProvided::Provided)
        } else {
          exports_info.is_export_provided(exports_info_artifact, &ids[1..])
        }
      } else {
        exports_info.is_export_provided(exports_info_artifact, ids)
      }
    }
    ExportsType::Namespace => {
      if first == "__esModule" {
        if ids.len() == 1 {
          Some(ExportProvided::Provided)
        } else {
          None
        }
      } else {
        exports_info.is_export_provided(exports_info_artifact, ids)
      }
    }
    ExportsType::Dynamic => {
      if first != "default" {
        exports_info.is_export_provided(exports_info_artifact, ids)
      } else {
        None
      }
    }
    ExportsType::DefaultOnly => None,
  }?;

  match provided {
    ExportProvided::Provided => Some(true),
    ExportProvided::NotProvided => Some(false),
    ExportProvided::Unknown => None,
  }
}
