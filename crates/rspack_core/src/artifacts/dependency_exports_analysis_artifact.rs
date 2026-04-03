use std::{collections::BTreeSet, sync::Arc};

use rspack_collections::{IdentifierMap, IdentifierSet};

use crate::{
  ArtifactExt, DeferredReexportSpec, DependencyId, ExportsSpec, ModuleIdentifier,
  incremental::{Incremental, IncrementalPasses},
};

#[derive(Debug, Default)]
pub struct ModuleDependencyExportsAnalysis {
  dirty: bool,
  dependency_ids: Arc<[DependencyId]>,
  targets: Vec<ModuleIdentifier>,
  flat_dependency_targets: Vec<ModuleIdentifier>,
  flat_local_apply: Vec<(DependencyId, ExportsSpec)>,
  structured_local_apply: Vec<(DependencyId, ExportsSpec)>,
  deferred_reexports: Vec<DeferredReexportSpec>,
}

impl ModuleDependencyExportsAnalysis {
  pub fn dirty(&self) -> bool {
    self.dirty
  }

  pub fn set_dirty(&mut self, dirty: bool) {
    self.dirty = dirty;
  }

  pub fn with_targets(targets: impl IntoIterator<Item = ModuleIdentifier>) -> Self {
    Self {
      targets: targets.into_iter().collect(),
      ..Default::default()
    }
  }

  pub fn with_staged_analysis(
    dependency_ids: Arc<[DependencyId]>,
    targets: impl IntoIterator<Item = ModuleIdentifier>,
    flat_dependency_targets: impl IntoIterator<Item = ModuleIdentifier>,
    flat_local_apply: impl IntoIterator<Item = (DependencyId, ExportsSpec)>,
    structured_local_apply: impl IntoIterator<Item = (DependencyId, ExportsSpec)>,
    deferred_reexports: impl IntoIterator<Item = DeferredReexportSpec>,
  ) -> Self {
    Self {
      dependency_ids,
      targets: targets.into_iter().collect(),
      flat_dependency_targets: flat_dependency_targets.into_iter().collect(),
      flat_local_apply: flat_local_apply.into_iter().collect(),
      structured_local_apply: structured_local_apply.into_iter().collect(),
      deferred_reexports: deferred_reexports.into_iter().collect(),
      ..Default::default()
    }
  }

  pub fn flat_local_apply(&self) -> &[(DependencyId, ExportsSpec)] {
    &self.flat_local_apply
  }

  pub fn dependency_ids(&self) -> &[DependencyId] {
    &self.dependency_ids
  }

  pub fn dependency_ids_arc(&self) -> &Arc<[DependencyId]> {
    &self.dependency_ids
  }

  pub fn targets(&self) -> &[ModuleIdentifier] {
    &self.targets
  }

  pub fn flat_dependency_targets(&self) -> &[ModuleIdentifier] {
    &self.flat_dependency_targets
  }

  pub fn structured_local_apply(&self) -> &[(DependencyId, ExportsSpec)] {
    &self.structured_local_apply
  }

  pub fn deferred_reexports(&self) -> &[DeferredReexportSpec] {
    &self.deferred_reexports
  }

  pub fn can_reuse_without_recollect(&self) -> bool {
    self.targets.is_empty()
      && self.structured_local_apply.is_empty()
      && self.deferred_reexports.is_empty()
      && self.flat_local_apply.iter().all(|(_, exports_spec)| {
        exports_spec.from.is_none()
          && exports_spec
            .dependencies
            .as_ref()
            .is_none_or(|dependencies| dependencies.is_empty())
      })
  }
}

#[derive(Debug, Default, Clone)]
struct DependencyExportsSccNode {
  modules: Vec<ModuleIdentifier>,
  incoming_sccs: Vec<usize>,
  outgoing_sccs: Vec<usize>,
  has_deferred_reexports: bool,
}

#[derive(Debug, Default, Clone)]
pub struct DependencyExportsTopology {
  scc_nodes: Vec<DependencyExportsSccNode>,
  waves: Vec<Vec<usize>>,
  wave_modules: Vec<IdentifierSet>,
  deferred_waves: Vec<Vec<usize>>,
}

#[derive(Debug, Default)]
pub struct DependencyExportsAnalysisArtifact {
  modules: IdentifierMap<ModuleDependencyExportsAnalysis>,
  flat_target_dependents: IdentifierMap<IdentifierSet>,
  topology: DependencyExportsTopology,
  topology_dirty: bool,
}

impl ArtifactExt for DependencyExportsAnalysisArtifact {
  const PASS: IncrementalPasses = IncrementalPasses::FINISH_MODULES;

  fn recover(incremental: &Incremental, new: &mut Self, old: &mut Self) {
    if incremental.mutations_readable(Self::PASS) {
      std::mem::swap(new, old);
      new.mark_all_dirty();
      new.set_topology_dirty(true);
    }
  }
}

impl DependencyExportsAnalysisArtifact {
  pub fn modules(&self) -> &IdentifierMap<ModuleDependencyExportsAnalysis> {
    &self.modules
  }

  pub fn module(
    &self,
    module_identifier: &ModuleIdentifier,
  ) -> Option<&ModuleDependencyExportsAnalysis> {
    self.modules.get(module_identifier)
  }

  pub fn flat_dependents_of(&self, module_identifier: &ModuleIdentifier) -> Option<&IdentifierSet> {
    self.flat_target_dependents.get(module_identifier)
  }

  pub fn module_mut(
    &mut self,
    module_identifier: &ModuleIdentifier,
  ) -> Option<&mut ModuleDependencyExportsAnalysis> {
    self.modules.get_mut(module_identifier)
  }

  pub fn dirty_modules(&self) -> IdentifierSet {
    self
      .modules
      .iter()
      .filter_map(|(module_identifier, analysis)| analysis.dirty().then_some(*module_identifier))
      .collect()
  }

  pub fn clear_all_dirty(&mut self) {
    self
      .modules
      .values_mut()
      .for_each(|analysis| analysis.set_dirty(false));
  }

  pub fn has_deferred_reexports(&self) -> bool {
    self
      .modules
      .values()
      .any(|analysis| !analysis.deferred_reexports().is_empty())
  }

  pub fn rebuild_topology(&mut self) {
    self.topology = DependencyExportsTopology::from_modules(&self.modules);
    self.topology_dirty = false;
  }

  pub fn replace_module(
    &mut self,
    module_identifier: ModuleIdentifier,
    analysis: ModuleDependencyExportsAnalysis,
  ) -> Option<ModuleDependencyExportsAnalysis> {
    let previous = self.modules.insert(module_identifier, analysis);
    self.update_flat_target_dependents(module_identifier, previous.as_ref());
    self.set_topology_dirty(true);
    previous
  }

  pub fn upsert_module(
    &mut self,
    module_identifier: ModuleIdentifier,
    analysis: ModuleDependencyExportsAnalysis,
  ) -> Option<ModuleDependencyExportsAnalysis> {
    let previous = self.modules.insert(module_identifier, analysis);
    self.update_flat_target_dependents(module_identifier, previous.as_ref());
    if previous
      .as_ref()
      .is_none_or(|previous| previous.targets() != self.modules[&module_identifier].targets())
    {
      self.set_topology_dirty(true);
    }
    previous
  }

  pub fn remove_module(
    &mut self,
    module_identifier: &ModuleIdentifier,
  ) -> Option<ModuleDependencyExportsAnalysis> {
    let previous = self.modules.remove(module_identifier);
    if previous.is_some() {
      self.remove_flat_target_dependents(*module_identifier, previous.as_ref());
      self.set_topology_dirty(true);
    }
    previous
  }

  fn mark_all_dirty(&mut self) {
    self
      .modules
      .values_mut()
      .for_each(|analysis| analysis.set_dirty(true));
  }

  pub fn topology_dirty(&self) -> bool {
    self.topology_dirty
  }

  pub fn topology(&self) -> &DependencyExportsTopology {
    &self.topology
  }

  fn set_topology_dirty(&mut self, topology_dirty: bool) {
    self.topology_dirty = topology_dirty;
  }

  fn update_flat_target_dependents(
    &mut self,
    module_identifier: ModuleIdentifier,
    previous: Option<&ModuleDependencyExportsAnalysis>,
  ) {
    self.remove_flat_target_dependents(module_identifier, previous);
    if let Some(analysis) = self.modules.get(&module_identifier) {
      for target in analysis.flat_dependency_targets() {
        self
          .flat_target_dependents
          .entry(*target)
          .or_default()
          .insert(module_identifier);
      }
    }
  }

  fn remove_flat_target_dependents(
    &mut self,
    module_identifier: ModuleIdentifier,
    previous: Option<&ModuleDependencyExportsAnalysis>,
  ) {
    let Some(previous) = previous else {
      return;
    };
    for target in previous.flat_dependency_targets() {
      let should_remove_entry =
        if let Some(dependents) = self.flat_target_dependents.get_mut(target) {
          dependents.remove(&module_identifier);
          dependents.is_empty()
        } else {
          false
        };
      if should_remove_entry {
        self.flat_target_dependents.remove(target);
      }
    }
  }
}

impl DependencyExportsTopology {
  fn from_modules(modules: &IdentifierMap<ModuleDependencyExportsAnalysis>) -> Self {
    let scc = compute_strongly_connected_components(modules);
    let scc_nodes = condense_scc_graph(modules, &scc);
    let waves = build_parallel_waves(&scc_nodes);
    let wave_modules = waves
      .iter()
      .map(|wave| {
        wave
          .iter()
          .flat_map(|scc_id| scc_nodes[*scc_id].modules.iter().copied())
          .collect()
      })
      .collect();
    let deferred_waves = waves
      .iter()
      .map(|wave| {
        wave
          .iter()
          .copied()
          .filter(|scc_id| scc_nodes[*scc_id].has_deferred_reexports)
          .collect()
      })
      .collect();

    Self {
      scc_nodes,
      waves,
      wave_modules,
      deferred_waves,
    }
  }

  pub fn scc_modules(&self, scc_id: usize) -> &[ModuleIdentifier] {
    self
      .scc_nodes
      .get(scc_id)
      .map_or(&[], |node| node.modules.as_slice())
  }

  pub fn waves(&self) -> &[Vec<usize>] {
    &self.waves
  }

  pub fn wave_modules(&self, wave_index: usize) -> &IdentifierSet {
    &self.wave_modules[wave_index]
  }

  pub fn deferred_wave(&self, wave_index: usize) -> &[usize] {
    &self.deferred_waves[wave_index]
  }
}

#[derive(Debug)]
struct StronglyConnectedComponents {
  module_to_scc: IdentifierMap<usize>,
  scc_modules: Vec<Vec<ModuleIdentifier>>,
}

#[derive(Debug)]
struct DfsFrame {
  module: ModuleIdentifier,
  next_neighbor_index: usize,
}

fn compute_strongly_connected_components(
  modules: &IdentifierMap<ModuleDependencyExportsAnalysis>,
) -> StronglyConnectedComponents {
  let module_graph = build_module_graph(modules);
  let reverse_module_graph = build_reverse_module_graph(&module_graph);
  let mut finish_order = build_finish_order(&module_graph);
  let mut visited = BTreeSet::new();
  let mut scc_modules = Vec::new();

  while let Some(module_identifier) = finish_order.pop() {
    if !visited.insert(module_identifier) {
      continue;
    }

    let mut component = Vec::new();
    let mut stack = vec![module_identifier];
    while let Some(module_identifier) = stack.pop() {
      component.push(module_identifier);

      for neighbor in reverse_module_graph
        .get(&module_identifier)
        .map_or(&[][..], Vec::as_slice)
        .iter()
        .rev()
      {
        if visited.insert(*neighbor) {
          stack.push(*neighbor);
        }
      }
    }

    component.sort_unstable();
    scc_modules.push(component);
  }

  scc_modules.sort_unstable();
  let mut module_to_scc = IdentifierMap::default();
  for (scc_id, modules) in scc_modules.iter().enumerate() {
    for module_identifier in modules {
      module_to_scc.insert(*module_identifier, scc_id);
    }
  }

  StronglyConnectedComponents {
    module_to_scc,
    scc_modules,
  }
}

fn build_module_graph(
  modules: &IdentifierMap<ModuleDependencyExportsAnalysis>,
) -> IdentifierMap<Vec<ModuleIdentifier>> {
  let mut module_identifiers = modules.keys().copied().collect::<Vec<_>>();
  module_identifiers.sort_unstable();

  module_identifiers
    .into_iter()
    .map(|module_identifier| {
      (
        module_identifier,
        module_targets(modules, module_identifier),
      )
    })
    .collect()
}

fn build_reverse_module_graph(
  module_graph: &IdentifierMap<Vec<ModuleIdentifier>>,
) -> IdentifierMap<Vec<ModuleIdentifier>> {
  let mut reverse_module_graph = IdentifierMap::default();
  let mut module_identifiers = module_graph.keys().copied().collect::<Vec<_>>();
  module_identifiers.sort_unstable();

  for module_identifier in &module_identifiers {
    reverse_module_graph.insert(*module_identifier, Vec::new());
  }

  for module_identifier in module_identifiers {
    for target in &module_graph[&module_identifier] {
      reverse_module_graph
        .get_mut(target)
        .expect("reverse graph should contain every target module")
        .push(module_identifier);
    }
  }

  reverse_module_graph.values_mut().for_each(|neighbors| {
    neighbors.sort_unstable();
    neighbors.dedup();
  });

  reverse_module_graph
}

fn build_finish_order(
  module_graph: &IdentifierMap<Vec<ModuleIdentifier>>,
) -> Vec<ModuleIdentifier> {
  let mut module_identifiers = module_graph.keys().copied().collect::<Vec<_>>();
  module_identifiers.sort_unstable();
  let mut visited = BTreeSet::new();
  let mut finish_order = Vec::with_capacity(module_identifiers.len());

  for module_identifier in module_identifiers {
    if !visited.insert(module_identifier) {
      continue;
    }

    let mut stack = vec![DfsFrame {
      module: module_identifier,
      next_neighbor_index: 0,
    }];

    while let Some(frame) = stack.last_mut() {
      let neighbors = module_graph
        .get(&frame.module)
        .map_or(&[][..], Vec::as_slice);

      if frame.next_neighbor_index < neighbors.len() {
        let neighbor = neighbors[frame.next_neighbor_index];
        frame.next_neighbor_index += 1;
        if visited.insert(neighbor) {
          stack.push(DfsFrame {
            module: neighbor,
            next_neighbor_index: 0,
          });
        }
      } else {
        let frame = stack.pop().expect("last frame should exist");
        finish_order.push(frame.module);
      }
    }
  }

  finish_order
}

fn condense_scc_graph(
  modules: &IdentifierMap<ModuleDependencyExportsAnalysis>,
  scc: &StronglyConnectedComponents,
) -> Vec<DependencyExportsSccNode> {
  let mut scc_nodes = scc
    .scc_modules
    .iter()
    .map(|scc_modules| DependencyExportsSccNode {
      modules: scc_modules.clone(),
      incoming_sccs: Vec::new(),
      outgoing_sccs: Vec::new(),
      has_deferred_reexports: scc_modules.iter().any(|module_identifier| {
        modules
          .get(module_identifier)
          .is_some_and(|analysis| !analysis.deferred_reexports().is_empty())
      }),
    })
    .collect::<Vec<_>>();
  let mut incoming_edges = vec![BTreeSet::new(); scc_nodes.len()];
  let mut outgoing_edges = vec![BTreeSet::new(); scc_nodes.len()];
  let mut module_identifiers = modules.keys().copied().collect::<Vec<_>>();
  module_identifiers.sort_unstable();

  for module_identifier in module_identifiers {
    let source_scc = scc.module_to_scc[&module_identifier];
    for target in module_targets(modules, module_identifier) {
      let target_scc = scc.module_to_scc[&target];
      if source_scc == target_scc {
        continue;
      }
      outgoing_edges[source_scc].insert(target_scc);
      incoming_edges[target_scc].insert(source_scc);
    }
  }

  for (scc_id, node) in scc_nodes.iter_mut().enumerate() {
    node.incoming_sccs = incoming_edges[scc_id].iter().copied().collect();
    node.outgoing_sccs = outgoing_edges[scc_id].iter().copied().collect();
  }

  scc_nodes
}

fn build_parallel_waves(scc_nodes: &[DependencyExportsSccNode]) -> Vec<Vec<usize>> {
  let mut remaining_outgoing = scc_nodes
    .iter()
    .map(|node| node.outgoing_sccs.len())
    .collect::<Vec<_>>();
  let mut processed = vec![false; scc_nodes.len()];
  let mut ready = remaining_outgoing
    .iter()
    .enumerate()
    .filter_map(|(scc_id, outgoing_count)| (*outgoing_count == 0).then_some(scc_id))
    .collect::<BTreeSet<_>>();
  let mut waves = Vec::new();

  while !ready.is_empty() {
    let wave = ready.iter().copied().collect::<Vec<_>>();
    ready.clear();

    for scc_id in &wave {
      processed[*scc_id] = true;
    }

    for scc_id in &wave {
      for incoming_scc in &scc_nodes[*scc_id].incoming_sccs {
        if processed[*incoming_scc] {
          continue;
        }
        remaining_outgoing[*incoming_scc] -= 1;
        if remaining_outgoing[*incoming_scc] == 0 {
          ready.insert(*incoming_scc);
        }
      }
    }

    waves.push(wave);
  }

  debug_assert!(processed.into_iter().all(|done| done));
  waves
}

fn module_targets(
  modules: &IdentifierMap<ModuleDependencyExportsAnalysis>,
  module_identifier: ModuleIdentifier,
) -> Vec<ModuleIdentifier> {
  let mut targets = modules
    .get(&module_identifier)
    .map(|analysis| {
      analysis
        .targets
        .iter()
        .copied()
        .filter(|target| modules.contains_key(target))
        .collect::<Vec<_>>()
    })
    .unwrap_or_default();

  targets.sort_unstable();
  targets.dedup();
  targets
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    ModuleIdentifier,
    incremental::{Incremental, IncrementalOptions, IncrementalPasses},
  };

  fn topology_wave_modules(
    topology: &DependencyExportsTopology,
  ) -> Vec<Vec<Vec<ModuleIdentifier>>> {
    topology
      .waves()
      .iter()
      .map(|wave| {
        wave
          .iter()
          .map(|scc_id| topology.scc_modules(*scc_id).to_vec())
          .collect()
      })
      .collect()
  }

  #[test]
  fn recover_keeps_previous_finish_modules_state_and_marks_it_dirty() {
    let module = ModuleIdentifier::from("module-a");
    let mut old = DependencyExportsAnalysisArtifact::default();
    old.replace_module(module, ModuleDependencyExportsAnalysis::default());
    old.set_topology_dirty(false);

    let incremental = Incremental::new_hot(IncrementalOptions {
      silent: true,
      passes: IncrementalPasses::FINISH_MODULES,
    });

    let mut new = DependencyExportsAnalysisArtifact::default();
    DependencyExportsAnalysisArtifact::recover(&incremental, &mut new, &mut old);

    assert!(new.modules().contains_key(&module));
    assert!(
      new
        .modules()
        .get(&module)
        .expect("module should recover")
        .dirty()
    );
    assert!(new.topology_dirty());
  }

  #[test]
  fn module_mutations_mark_topology_dirty() {
    let module = ModuleIdentifier::from("module-a");
    let mut artifact = DependencyExportsAnalysisArtifact::default();

    artifact.set_topology_dirty(false);
    artifact.replace_module(module, ModuleDependencyExportsAnalysis::default());
    assert!(artifact.topology_dirty());

    artifact.set_topology_dirty(false);
    artifact.remove_module(&module);
    assert!(artifact.topology_dirty());
  }

  #[test]
  fn rebuild_topology_groups_independent_sccs_into_the_same_wave() {
    let mut artifact = DependencyExportsAnalysisArtifact::default();
    let a = ModuleIdentifier::from("a");
    let b = ModuleIdentifier::from("b");
    let y = ModuleIdentifier::from("y");
    let z = ModuleIdentifier::from("z");

    artifact.replace_module(a, ModuleDependencyExportsAnalysis::with_targets([z]));
    artifact.replace_module(b, ModuleDependencyExportsAnalysis::with_targets([y]));
    artifact.replace_module(y, ModuleDependencyExportsAnalysis::default());
    artifact.replace_module(z, ModuleDependencyExportsAnalysis::default());

    artifact.rebuild_topology();

    assert_eq!(
      topology_wave_modules(artifact.topology()),
      vec![vec![vec![y], vec![z]], vec![vec![a], vec![b]]]
    );
  }

  #[test]
  fn rebuild_topology_puts_a_cycle_into_one_scc() {
    let mut artifact = DependencyExportsAnalysisArtifact::default();
    let a = ModuleIdentifier::from("cycle-a");
    let b = ModuleIdentifier::from("cycle-b");

    artifact.replace_module(a, ModuleDependencyExportsAnalysis::with_targets([b]));
    artifact.replace_module(b, ModuleDependencyExportsAnalysis::with_targets([a]));

    artifact.rebuild_topology();

    assert_eq!(
      topology_wave_modules(artifact.topology()),
      vec![vec![vec![a, b]]]
    );
  }
}
