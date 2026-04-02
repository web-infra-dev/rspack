use std::collections::BTreeSet;

use rspack_collections::{IdentifierMap, IdentifierSet};

use crate::{
  ArtifactExt, ModuleIdentifier,
  incremental::{Incremental, IncrementalPasses},
};

#[derive(Debug, Default, Clone)]
pub struct ModuleDependencyExportsAnalysis {
  dirty: bool,
  targets: Vec<ModuleIdentifier>,
}

impl ModuleDependencyExportsAnalysis {
  pub fn dirty(&self) -> bool {
    self.dirty
  }

  pub fn set_dirty(&mut self, dirty: bool) {
    self.dirty = dirty;
  }

  #[cfg(test)]
  fn with_targets(targets: impl IntoIterator<Item = ModuleIdentifier>) -> Self {
    Self {
      targets: targets.into_iter().collect(),
      ..Default::default()
    }
  }
}

#[derive(Debug, Default, Clone)]
pub struct DependencyExportsSccNode {
  modules: Vec<ModuleIdentifier>,
  incoming_sccs: Vec<usize>,
  outgoing_sccs: Vec<usize>,
}

#[derive(Debug, Default, Clone)]
pub struct DependencyExportsTopology {
  module_to_scc: IdentifierMap<usize>,
  scc_nodes: Vec<DependencyExportsSccNode>,
  waves: Vec<Vec<usize>>,
}

#[derive(Debug, Default, Clone)]
pub struct DependencyExportsAnalysisArtifact {
  modules: IdentifierMap<ModuleDependencyExportsAnalysis>,
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

  pub fn rebuild_topology(&mut self) {
    self.topology = DependencyExportsTopology::from_modules(&self.modules);
    self.topology_dirty = false;
  }

  #[cfg(test)]
  fn replace_module(
    &mut self,
    module_identifier: ModuleIdentifier,
    analysis: ModuleDependencyExportsAnalysis,
  ) -> Option<ModuleDependencyExportsAnalysis> {
    let previous = self.modules.insert(module_identifier, analysis);
    self.set_topology_dirty(true);
    previous
  }

  #[cfg(test)]
  fn remove_module(
    &mut self,
    module_identifier: &ModuleIdentifier,
  ) -> Option<ModuleDependencyExportsAnalysis> {
    let previous = self.modules.remove(module_identifier);
    if previous.is_some() {
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
}

impl DependencyExportsTopology {
  fn from_modules(modules: &IdentifierMap<ModuleDependencyExportsAnalysis>) -> Self {
    let scc = tarjan_scc(modules);
    let scc_nodes = condense_scc_graph(modules, &scc);
    let waves = build_parallel_waves(&scc_nodes);

    Self {
      module_to_scc: scc.module_to_scc,
      scc_nodes,
      waves,
    }
  }

  pub fn module_to_scc(&self) -> &IdentifierMap<usize> {
    &self.module_to_scc
  }

  pub fn scc_nodes(&self) -> &[DependencyExportsSccNode] {
    &self.scc_nodes
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
}

#[derive(Debug)]
struct StronglyConnectedComponents {
  module_to_scc: IdentifierMap<usize>,
  scc_modules: Vec<Vec<ModuleIdentifier>>,
}

#[derive(Debug)]
struct TarjanState<'a> {
  modules: &'a IdentifierMap<ModuleDependencyExportsAnalysis>,
  next_index: usize,
  indices: IdentifierMap<usize>,
  lowlinks: IdentifierMap<usize>,
  stack: Vec<ModuleIdentifier>,
  on_stack: IdentifierSet,
  module_to_scc: IdentifierMap<usize>,
  scc_modules: Vec<Vec<ModuleIdentifier>>,
}

impl TarjanState<'_> {
  fn strong_connect(&mut self, module_identifier: ModuleIdentifier) {
    let module_index = self.next_index;
    self.next_index += 1;
    self.indices.insert(module_identifier, module_index);
    self.lowlinks.insert(module_identifier, module_index);
    self.stack.push(module_identifier);
    self.on_stack.insert(module_identifier);

    for target in module_targets(self.modules, module_identifier) {
      if !self.indices.contains_key(&target) {
        self.strong_connect(target);
        let target_lowlink = *self
          .lowlinks
          .get(&target)
          .expect("target lowlink should exist after traversal");
        let lowlink = self
          .lowlinks
          .get_mut(&module_identifier)
          .expect("module lowlink should exist");
        *lowlink = (*lowlink).min(target_lowlink);
      } else if self.on_stack.contains(&target) {
        let target_index = *self
          .indices
          .get(&target)
          .expect("stacked target index should exist");
        let lowlink = self
          .lowlinks
          .get_mut(&module_identifier)
          .expect("module lowlink should exist");
        *lowlink = (*lowlink).min(target_index);
      }
    }

    if self.lowlinks[&module_identifier] != module_index {
      return;
    }

    let mut component = Vec::new();
    loop {
      let stack_module = self
        .stack
        .pop()
        .expect("component root should have at least one stack item");
      self.on_stack.remove(&stack_module);
      component.push(stack_module);
      if stack_module == module_identifier {
        break;
      }
    }

    component.sort_unstable();
    let scc_id = self.scc_modules.len();
    for module in &component {
      self.module_to_scc.insert(*module, scc_id);
    }
    self.scc_modules.push(component);
  }
}

fn tarjan_scc(
  modules: &IdentifierMap<ModuleDependencyExportsAnalysis>,
) -> StronglyConnectedComponents {
  let mut module_identifiers = modules.keys().copied().collect::<Vec<_>>();
  module_identifiers.sort_unstable();

  let mut state = TarjanState {
    modules,
    next_index: 0,
    indices: IdentifierMap::default(),
    lowlinks: IdentifierMap::default(),
    stack: Vec::new(),
    on_stack: IdentifierSet::default(),
    module_to_scc: IdentifierMap::default(),
    scc_modules: Vec::new(),
  };

  for module_identifier in module_identifiers {
    if !state.indices.contains_key(&module_identifier) {
      state.strong_connect(module_identifier);
    }
  }

  StronglyConnectedComponents {
    module_to_scc: state.module_to_scc,
    scc_modules: state.scc_modules,
  }
}

fn condense_scc_graph(
  modules: &IdentifierMap<ModuleDependencyExportsAnalysis>,
  scc: &StronglyConnectedComponents,
) -> Vec<DependencyExportsSccNode> {
  let mut scc_nodes = scc
    .scc_modules
    .iter()
    .map(|modules| DependencyExportsSccNode {
      modules: modules.clone(),
      incoming_sccs: Vec::new(),
      outgoing_sccs: Vec::new(),
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
    let c = ModuleIdentifier::from("c");
    let d = ModuleIdentifier::from("d");

    artifact.replace_module(a, ModuleDependencyExportsAnalysis::with_targets([c]));
    artifact.replace_module(b, ModuleDependencyExportsAnalysis::with_targets([d]));
    artifact.replace_module(c, ModuleDependencyExportsAnalysis::default());
    artifact.replace_module(d, ModuleDependencyExportsAnalysis::default());

    artifact.rebuild_topology();

    assert_eq!(artifact.topology().waves().len(), 2);
    assert_eq!(artifact.topology().waves()[0].len(), 2);
  }

  #[test]
  fn rebuild_topology_puts_a_cycle_into_one_scc() {
    let mut artifact = DependencyExportsAnalysisArtifact::default();
    let a = ModuleIdentifier::from("cycle-a");
    let b = ModuleIdentifier::from("cycle-b");

    artifact.replace_module(a, ModuleDependencyExportsAnalysis::with_targets([b]));
    artifact.replace_module(b, ModuleDependencyExportsAnalysis::with_targets([a]));

    artifact.rebuild_topology();

    assert_eq!(artifact.topology().scc_nodes().len(), 1);
  }
}
