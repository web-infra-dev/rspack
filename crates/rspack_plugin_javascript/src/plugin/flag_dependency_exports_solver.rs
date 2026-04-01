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
  pub is_cyclic: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SolveGraph {
  pub components: Vec<SolveComponent>,
  pub reverse_topo_order: Vec<usize>,
  pub module_to_component: IdentifierMap<usize>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct ExportsSolveStats {
  pub planned_modules: usize,
  pub provider_edges: usize,
  pub scc_count: usize,
  pub fallback_components: usize,
  pub solve_module_once_calls: usize,
  pub local_requeues: usize,
}

#[derive(Debug)]
struct TarjanState {
  index: usize,
  stack: Vec<ModuleIdentifier>,
  on_stack: IdentifierSet,
  indices: IdentifierMap<usize>,
  low_links: IdentifierMap<usize>,
  components: Vec<Vec<ModuleIdentifier>>,
}

pub(crate) fn build_solve_graph(planned_modules: Vec<PlannedModule>) -> SolveGraph {
  let module_index = planned_modules
    .iter()
    .enumerate()
    .map(|(index, planned)| (planned.module_id, index))
    .collect::<IdentifierMap<_>>();
  let mut components = condense_components(&planned_modules, &module_index);
  components = group_fallback_components(components, &planned_modules, &module_index);
  let module_to_component = components
    .iter()
    .enumerate()
    .flat_map(|(component_id, component)| {
      component
        .modules
        .iter()
        .copied()
        .map(move |module_id| (module_id, component_id))
    })
    .collect::<IdentifierMap<_>>();
  let reverse_topo_order =
    reverse_topological_component_order(&planned_modules, &components, &module_to_component);

  SolveGraph {
    components,
    reverse_topo_order,
    module_to_component,
  }
}

fn condense_components(
  planned_modules: &[PlannedModule],
  module_index: &IdentifierMap<usize>,
) -> Vec<SolveComponent> {
  let mut tarjan = TarjanState {
    index: 0,
    stack: Vec::new(),
    on_stack: IdentifierSet::default(),
    indices: IdentifierMap::default(),
    low_links: IdentifierMap::default(),
    components: Vec::new(),
  };

  for planned in planned_modules {
    if !tarjan.indices.contains_key(&planned.module_id) {
      strong_connect(
        planned.module_id,
        planned_modules,
        module_index,
        &mut tarjan,
      );
    }
  }

  tarjan
    .components
    .into_iter()
    .map(|modules| build_component(modules, planned_modules, module_index))
    .collect()
}

fn build_component(
  modules: Vec<ModuleIdentifier>,
  planned_modules: &[PlannedModule],
  module_index: &IdentifierMap<usize>,
) -> SolveComponent {
  let module_set = modules.iter().copied().collect::<IdentifierSet>();
  let mut dependents_within_component = modules
    .iter()
    .copied()
    .map(|module_id| (module_id, IdentifierSet::default()))
    .collect::<IdentifierMap<_>>();
  let mut fallback = false;
  let mut is_cyclic = false;

  for module_id in modules.iter().copied() {
    let planned = &planned_modules[module_index[&module_id]];
    if planned.has_unknown_provider {
      fallback = true;
    }
    if planned.provider_modules.contains(&module_id) {
      is_cyclic = true;
    }
    for provider in &planned.provider_modules {
      if !module_set.contains(provider) {
        continue;
      }
      if let Some(dependents) = dependents_within_component.get_mut(provider) {
        dependents.insert(module_id);
      }
    }
  }

  if modules.len() > 1 {
    is_cyclic = true;
  }

  SolveComponent {
    modules,
    dependents_within_component,
    fallback,
    is_cyclic,
  }
}

fn group_fallback_components(
  components: Vec<SolveComponent>,
  planned_modules: &[PlannedModule],
  module_index: &IdentifierMap<usize>,
) -> Vec<SolveComponent> {
  let component_count = components.len();
  let fallback_roots = components
    .iter()
    .enumerate()
    .filter_map(|(index, component)| component.fallback.then_some(index))
    .collect::<Vec<_>>();

  if fallback_roots.is_empty() {
    return components;
  }

  let module_to_component = components
    .iter()
    .enumerate()
    .flat_map(|(component_id, component)| {
      component
        .modules
        .iter()
        .copied()
        .map(move |module_id| (module_id, component_id))
    })
    .collect::<IdentifierMap<_>>();

  let mut outgoing = vec![Vec::new(); component_count];
  for planned in planned_modules {
    let consumer = module_to_component[&planned.module_id];
    for provider in &planned.provider_modules {
      let Some(&provider_component) = module_to_component.get(provider) else {
        continue;
      };
      if consumer == provider_component {
        continue;
      }
      outgoing[provider_component].push(consumer);
    }
  }

  let mut parent = (0..component_count).collect::<Vec<_>>();
  for root in &fallback_roots {
    let mut stack = Vec::from([*root]);
    let mut visited = vec![false; component_count];
    visited[*root] = true;

    while let Some(component_id) = stack.pop() {
      for &dependent in &outgoing[component_id] {
        if !visited[dependent] {
          visited[dependent] = true;
          union(&mut parent, *root, dependent);
          stack.push(dependent);
        }
      }
    }
  }

  let mut grouped_components = vec![Vec::new(); component_count];
  for component_id in 0..component_count {
    let root = find(&mut parent, component_id);
    grouped_components[root].push(component_id);
  }

  grouped_components
    .into_iter()
    .filter(|indices| !indices.is_empty())
    .map(|indices| {
      let modules = indices
        .into_iter()
        .flat_map(|index| components[index].modules.iter().copied())
        .collect::<Vec<_>>();
      build_component(modules, planned_modules, module_index)
    })
    .collect()
}

fn find(parent: &mut Vec<usize>, component_id: usize) -> usize {
  if parent[component_id] != component_id {
    let root = find(parent, parent[component_id]);
    parent[component_id] = root;
  }
  parent[component_id]
}

fn union(parent: &mut Vec<usize>, left: usize, right: usize) {
  let left = find(parent, left);
  let right = find(parent, right);
  parent[right] = left;
}

fn strong_connect(
  module_id: ModuleIdentifier,
  planned_modules: &[PlannedModule],
  module_index: &IdentifierMap<usize>,
  tarjan: &mut TarjanState,
) {
  tarjan.indices.insert(module_id, tarjan.index);
  tarjan.low_links.insert(module_id, tarjan.index);
  tarjan.index += 1;
  tarjan.stack.push(module_id);
  tarjan.on_stack.insert(module_id);

  let planned = &planned_modules[module_index[&module_id]];
  for provider in &planned.provider_modules {
    if !module_index.contains_key(provider) {
      continue;
    }
    if !tarjan.indices.contains_key(provider) {
      strong_connect(*provider, planned_modules, module_index, tarjan);
      let low_link = tarjan.low_links[&module_id].min(tarjan.low_links[provider]);
      tarjan.low_links.insert(module_id, low_link);
    } else if tarjan.on_stack.contains(provider) {
      let low_link = tarjan.low_links[&module_id].min(tarjan.indices[provider]);
      tarjan.low_links.insert(module_id, low_link);
    }
  }

  if tarjan.low_links[&module_id] == tarjan.indices[&module_id] {
    let mut component = Vec::new();
    while let Some(popped) = tarjan.stack.pop() {
      tarjan.on_stack.remove(&popped);
      component.push(popped);
      if popped == module_id {
        break;
      }
    }
    tarjan.components.push(component);
  }
}

fn reverse_topological_component_order(
  planned_modules: &[PlannedModule],
  components: &[SolveComponent],
  module_to_component: &IdentifierMap<usize>,
) -> Vec<usize> {
  let mut indegree = vec![0usize; components.len()];
  let mut outgoing = vec![Vec::new(); components.len()];

  for planned in planned_modules {
    let consumer = module_to_component[&planned.module_id];
    for provider in &planned.provider_modules {
      let Some(&provider_component) = module_to_component.get(provider) else {
        continue;
      };
      if consumer == provider_component {
        continue;
      }
      outgoing[provider_component].push(consumer);
      indegree[consumer] += 1;
    }
  }

  let mut queue = VecDeque::from(
    indegree
      .iter()
      .enumerate()
      .filter_map(|(index, degree)| (*degree == 0).then_some(index))
      .collect::<Vec<_>>(),
  );
  let mut order = Vec::with_capacity(components.len());

  while let Some(component_id) = queue.pop_front() {
    order.push(component_id);
    for dependent in &outgoing[component_id] {
      indegree[*dependent] -= 1;
      if indegree[*dependent] == 0 {
        queue.push_back(*dependent);
      }
    }
  }

  order
}

pub(crate) fn execute_component_worklist(
  component: &SolveComponent,
  queue: &mut VecDeque<ModuleIdentifier>,
  mut solve_once: impl FnMut(ModuleIdentifier) -> bool,
) {
  if queue.is_empty() {
    queue.extend(component.modules.iter().copied());
  }

  while let Some(module_id) = queue.pop_front() {
    if solve_once(module_id) {
      if let Some(dependents) = component.dependents_within_component.get(&module_id) {
        queue.extend(dependents.iter().copied());
      }
    }
  }
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

  fn planned_with_unknown(
    name: &str,
    providers: &[&str],
    has_unknown_provider: bool,
  ) -> PlannedModule {
    PlannedModule {
      module_id: module(name),
      provider_modules: providers
        .iter()
        .map(|provider| module(provider))
        .collect::<IdentifierSet>(),
      has_unknown_provider,
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
  fn build_solve_graph_marks_singleton_self_cycle_as_cyclic() {
    let graph = build_solve_graph(vec![planned_with_unknown("a", &["a"], false)]);

    assert_eq!(graph.components.len(), 1);
    assert_eq!(graph.components[0].modules, vec![module("a")]);
    assert!(graph.components[0].is_cyclic);
  }

  #[test]
  fn build_solve_graph_groups_unknown_providers_into_fallback_component() {
    let graph = build_solve_graph(vec![
      planned_with_unknown("leaf", &[], false),
      planned_with_unknown("unknown", &["leaf"], true),
      planned("consumer", &["unknown"]),
      planned("sink", &["consumer"]),
    ]);

    let unknown_component = graph.module_to_component[&module("unknown")];
    let consumer_component = graph.module_to_component[&module("consumer")];
    let sink_component = graph.module_to_component[&module("sink")];
    let leaf_component = graph.module_to_component[&module("leaf")];

    assert_eq!(graph.components.len(), 2);
    assert_eq!(unknown_component, consumer_component);
    assert_eq!(unknown_component, sink_component);
    assert!(graph.components[unknown_component].fallback);
    assert!(graph.components[consumer_component].fallback);
    assert!(graph.components[sink_component].fallback);
    assert_ne!(unknown_component, leaf_component);
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
      is_cyclic: false,
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
