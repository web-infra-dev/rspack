use std::collections::VecDeque;

use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::ModuleIdentifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PlannedModule {
  pub module_id: ModuleIdentifier,
  pub provider_modules: IdentifierSet,
  pub has_unknown_provider: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SolveComponent {
  pub modules: Vec<ModuleIdentifier>,
  pub dependents_within_component: IdentifierMap<IdentifierSet>,
  pub fallback: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SolveGraph {
  pub components: Vec<SolveComponent>,
  pub reverse_topo_order: Vec<usize>,
  pub module_to_component: IdentifierMap<usize>,
}

#[cfg(test)]
mod tests {
  use super::*;

  fn module(name: &str) -> ModuleIdentifier {
    ModuleIdentifier::from(name)
  }

  fn planned(name: &str, providers: &[&str]) -> PlannedModule {
    PlannedModule {
      module_id: module(name),
      provider_modules: providers
        .iter()
        .map(|provider| module(provider))
        .collect::<IdentifierSet>(),
      has_unknown_provider: false,
    }
  }

  #[test]
  fn build_solve_graph_solves_providers_before_consumers() {
    let graph = build_solve_graph(vec![
      planned("entry", &["mid"]),
      planned("mid", &["leaf"]),
      planned("leaf", &[]),
    ]);

    let order = graph
      .reverse_topo_order
      .iter()
      .map(|component_id| graph.components[*component_id].modules[0])
      .collect::<Vec<_>>();

    assert_eq!(order, vec![module("leaf"), module("mid"), module("entry")]);
  }

  #[test]
  fn build_solve_graph_collapses_cycles_into_one_component() {
    let graph = build_solve_graph(vec![
      planned("a", &["b"]),
      planned("b", &["a"]),
      planned("leaf", &[]),
    ]);

    assert_eq!(graph.components.len(), 2);
    assert_eq!(
      graph.module_to_component[&module("a")],
      graph.module_to_component[&module("b")]
    );
  }

  #[test]
  fn execute_component_worklist_requeues_only_changed_in_component_dependents() {
    let component = SolveComponent {
      modules: vec![module("a"), module("b")],
      dependents_within_component: [
        (
          module("a"),
          [module("b")].into_iter().collect::<IdentifierSet>(),
        ),
        (
          module("b"),
          [module("a")].into_iter().collect::<IdentifierSet>(),
        ),
      ]
      .into_iter()
      .collect::<IdentifierMap<_>>(),
      fallback: false,
    };
    let mut queue = VecDeque::new();
    let mut calls = Vec::new();

    execute_component_worklist(&component, &mut queue, |module_id| {
      calls.push(module_id);
      module_id == module("b") && calls.len() == 2
    });

    assert_eq!(calls, vec![module("a"), module("b"), module("a")]);
  }
}
