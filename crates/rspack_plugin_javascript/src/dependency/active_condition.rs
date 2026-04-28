use rspack_cacheable::{
  cacheable,
  with::{AsCacheable, AsVec},
};
use rspack_core::{
  ConnectionState, Dependency, DependencyCondition, DependencyConditionFn, DependencyId,
  EvaluatedInlinableValue, ExportsInfoArtifact, ModuleGraph, ModuleGraphCacheArtifact,
  ModuleGraphConnection, RuntimeSpec, SideEffectsStateArtifact, UsedName,
};

use super::{CommonJsRequireDependency, ESMImportSpecifierDependency};

#[cacheable]
#[derive(Debug, Clone)]
pub enum DependencyActiveCondition {
  ESMImportedBoolean {
    dependency_id: DependencyId,
    expected: bool,
  },
  ESMImportedBooleanExpression {
    #[cacheable(with=AsVec<AsCacheable>)]
    nodes: Vec<ESMImportedBooleanExpressionNode>,
    root: u32,
  },
}

#[cacheable]
#[derive(Debug, Clone)]
pub enum ESMImportedBooleanExpressionNode {
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
pub struct DependencyActiveConditions {
  // Multiple entries are accumulated from nested branch guards, so the top-level list is conjunctive.
  // A single entry may contain its own expression tree for compound tests like `a && b` or `a || b`.
  #[cacheable(with=AsVec<AsCacheable>)]
  conditions: Vec<DependencyActiveCondition>,
}

impl DependencyActiveConditions {
  pub fn extend(&mut self, conditions: impl IntoIterator<Item = DependencyActiveCondition>) {
    self.conditions.extend(conditions);
  }

  fn is_empty(&self) -> bool {
    self.conditions.is_empty()
  }

  fn iter(&self) -> impl Iterator<Item = &DependencyActiveCondition> {
    self.conditions.iter()
  }
}

pub fn set_dependency_active_conditions(
  dep: &mut dyn Dependency,
  conditions: &[DependencyActiveCondition],
) {
  if conditions.is_empty() {
    return;
  }

  if let Some(dep) = dep.downcast_mut::<CommonJsRequireDependency>() {
    dep.add_active_conditions(conditions.iter().cloned());
  } else if let Some(dep) = dep.downcast_mut::<ESMImportSpecifierDependency>() {
    dep.add_active_conditions(conditions.iter().cloned());
  }
}

pub fn compose_dependency_condition(
  base: Option<DependencyCondition>,
  active_conditions: Option<&DependencyActiveConditions>,
) -> Option<DependencyCondition> {
  let Some(active_conditions) = active_conditions.filter(|conditions| !conditions.is_empty())
  else {
    return base;
  };

  Some(DependencyCondition::new(ActiveDependencyCondition {
    base,
    active_conditions: active_conditions.clone(),
  }))
}

struct ActiveDependencyCondition {
  base: Option<DependencyCondition>,
  active_conditions: DependencyActiveConditions,
}

impl DependencyConditionFn for ActiveDependencyCondition {
  fn get_connection_state(
    &self,
    conn: &ModuleGraphConnection,
    runtime: Option<&RuntimeSpec>,
    module_graph: &ModuleGraph,
    module_graph_cache: &ModuleGraphCacheArtifact,
    side_effects_state_artifact: &SideEffectsStateArtifact,
    exports_info_artifact: &ExportsInfoArtifact,
  ) -> ConnectionState {
    for condition in self.active_conditions.iter() {
      if matches!(
        resolve_active_condition(condition, runtime, module_graph, exports_info_artifact),
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

fn resolve_active_condition(
  condition: &DependencyActiveCondition,
  runtime: Option<&RuntimeSpec>,
  module_graph: &ModuleGraph,
  exports_info_artifact: &ExportsInfoArtifact,
) -> Option<bool> {
  match condition {
    DependencyActiveCondition::ESMImportedBoolean {
      dependency_id,
      expected,
    } => resolve_esm_imported_boolean_condition(
      dependency_id,
      *expected,
      runtime,
      module_graph,
      exports_info_artifact,
    ),
    DependencyActiveCondition::ESMImportedBooleanExpression { nodes, root } => {
      resolve_esm_imported_boolean_expression(
        nodes,
        *root,
        runtime,
        module_graph,
        exports_info_artifact,
      )
    }
  }
}

fn resolve_esm_imported_boolean_expression(
  nodes: &[ESMImportedBooleanExpressionNode],
  root: u32,
  runtime: Option<&RuntimeSpec>,
  module_graph: &ModuleGraph,
  exports_info_artifact: &ExportsInfoArtifact,
) -> Option<bool> {
  let node = nodes.get(root as usize)?;
  match node {
    ESMImportedBooleanExpressionNode::Constant(value) => Some(*value),
    ESMImportedBooleanExpressionNode::ESMImportedBoolean {
      dependency_id,
      expected,
    } => resolve_esm_imported_boolean_condition(
      dependency_id,
      *expected,
      runtime,
      module_graph,
      exports_info_artifact,
    ),
    ESMImportedBooleanExpressionNode::All { left, right } => {
      let left = resolve_esm_imported_boolean_expression(
        nodes,
        *left,
        runtime,
        module_graph,
        exports_info_artifact,
      );
      if matches!(left, Some(false)) {
        return Some(false);
      }
      let right = resolve_esm_imported_boolean_expression(
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
    ESMImportedBooleanExpressionNode::Any { left, right } => {
      let left = resolve_esm_imported_boolean_expression(
        nodes,
        *left,
        runtime,
        module_graph,
        exports_info_artifact,
      );
      if matches!(left, Some(true)) {
        return Some(true);
      }
      let right = resolve_esm_imported_boolean_expression(
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

fn resolve_esm_imported_boolean_condition(
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
