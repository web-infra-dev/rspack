use rspack_cacheable::{
  cacheable,
  with::{AsCacheable, AsVec},
};
use rspack_core::{
  ConnectionState, Dependency, DependencyCondition, DependencyConditionFn, DependencyId,
  EvaluatedInlinableValue, ExportsInfoArtifact, ModuleGraph, ModuleGraphCacheArtifact,
  ModuleGraphConnection, RuntimeSpec, SideEffectsStateArtifact, UsedName,
};

use super::{CommonJsRequireDependency, ESMImportSpecifierDependency, ImportDependency};

#[cacheable]
#[derive(Debug, Clone)]
pub enum DependencyBranchGuard {
  ESMImportedBoolean {
    dependency_id: DependencyId,
    expected: bool,
  },
  ESMImportedBooleanExpression {
    #[cacheable(with=AsVec<AsCacheable>)]
    nodes: Vec<ESMImportedBooleanGuardNode>,
    root: u32,
  },
}

#[cacheable]
#[derive(Debug, Clone)]
pub enum ESMImportedBooleanGuardNode {
  Constant(bool),
  ESMImportedBoolean {
    dependency_id: DependencyId,
    expected: bool,
  },
  All {
    left: u32,
    right: u32,
  },
  Any {
    left: u32,
    right: u32,
  },
}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct DependencyBranchGuards {
  // Multiple entries are accumulated from nested branch guards, so the top-level list is conjunctive.
  // A single entry may contain its own expression tree for compound tests like `a && b` or `a || b`.
  #[cacheable(with=AsVec<AsCacheable>)]
  guards: Vec<DependencyBranchGuard>,
}

impl DependencyBranchGuards {
  pub fn extend(&mut self, guards: impl IntoIterator<Item = DependencyBranchGuard>) {
    self.guards.extend(guards);
  }

  fn is_empty(&self) -> bool {
    self.guards.is_empty()
  }

  fn iter(&self) -> impl Iterator<Item = &DependencyBranchGuard> {
    self.guards.iter()
  }
}

pub fn set_dependency_branch_guards(dep: &mut dyn Dependency, guards: &[DependencyBranchGuard]) {
  if guards.is_empty() {
    return;
  }

  if let Some(dep) = dep.downcast_mut::<CommonJsRequireDependency>() {
    dep.add_branch_guards(guards.iter().cloned());
  } else if let Some(dep) = dep.downcast_mut::<ESMImportSpecifierDependency>() {
    dep.add_branch_guards(guards.iter().cloned());
  } else if let Some(dep) = dep.downcast_mut::<ImportDependency>() {
    dep.add_branch_guards(guards.iter().cloned());
  }
}

pub fn compose_dependency_condition(
  base: Option<DependencyCondition>,
  branch_guards: Option<&DependencyBranchGuards>,
) -> Option<DependencyCondition> {
  let Some(branch_guards) = branch_guards.filter(|guards| !guards.is_empty()) else {
    return base;
  };

  Some(DependencyCondition::new(BranchGuardDependencyCondition {
    base,
    branch_guards: branch_guards.clone(),
  }))
}

struct BranchGuardDependencyCondition {
  base: Option<DependencyCondition>,
  branch_guards: DependencyBranchGuards,
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
    for condition in self.branch_guards.iter() {
      if matches!(
        resolve_branch_guard(condition, runtime, module_graph, exports_info_artifact),
        Some(false)
      ) {
        return ConnectionState::Active(false);
      }
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
  match condition {
    DependencyBranchGuard::ESMImportedBoolean {
      dependency_id,
      expected,
    } => resolve_esm_imported_boolean_guard(
      dependency_id,
      *expected,
      runtime,
      module_graph,
      exports_info_artifact,
    ),
    DependencyBranchGuard::ESMImportedBooleanExpression { nodes, root } => {
      resolve_esm_imported_boolean_guard_expression(
        nodes,
        *root,
        runtime,
        module_graph,
        exports_info_artifact,
      )
    }
  }
}

fn resolve_esm_imported_boolean_guard_expression(
  nodes: &[ESMImportedBooleanGuardNode],
  root: u32,
  runtime: Option<&RuntimeSpec>,
  module_graph: &ModuleGraph,
  exports_info_artifact: &ExportsInfoArtifact,
) -> Option<bool> {
  let node = nodes.get(root as usize)?;
  match node {
    ESMImportedBooleanGuardNode::Constant(value) => Some(*value),
    ESMImportedBooleanGuardNode::ESMImportedBoolean {
      dependency_id,
      expected,
    } => resolve_esm_imported_boolean_guard(
      dependency_id,
      *expected,
      runtime,
      module_graph,
      exports_info_artifact,
    ),
    ESMImportedBooleanGuardNode::All { left, right } => {
      let left = resolve_esm_imported_boolean_guard_expression(
        nodes,
        *left,
        runtime,
        module_graph,
        exports_info_artifact,
      );
      if matches!(left, Some(false)) {
        return Some(false);
      }
      let right = resolve_esm_imported_boolean_guard_expression(
        nodes,
        *right,
        runtime,
        module_graph,
        exports_info_artifact,
      );
      match (left, right) {
        (Some(true), Some(true)) => Some(true),
        (Some(false), _) | (_, Some(false)) => Some(false),
        _ => None,
      }
    }
    ESMImportedBooleanGuardNode::Any { left, right } => {
      let left = resolve_esm_imported_boolean_guard_expression(
        nodes,
        *left,
        runtime,
        module_graph,
        exports_info_artifact,
      );
      if matches!(left, Some(true)) {
        return Some(true);
      }
      let right = resolve_esm_imported_boolean_guard_expression(
        nodes,
        *right,
        runtime,
        module_graph,
        exports_info_artifact,
      );
      match (left, right) {
        (Some(true), _) | (_, Some(true)) => Some(true),
        (Some(false), Some(false)) => Some(false),
        _ => None,
      }
    }
  }
}

fn resolve_esm_imported_boolean_guard(
  dependency_id: &DependencyId,
  expected: bool,
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
    EvaluatedInlinableValue::Boolean(value) => Some(*value == expected),
    _ => None,
  }
}
