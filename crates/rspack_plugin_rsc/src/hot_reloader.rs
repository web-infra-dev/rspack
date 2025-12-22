use std::hash::{Hash, Hasher};

use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::{Compilation, Module, ModuleGraphRef, RSCModuleType, RuntimeSpec};
use rustc_hash::{FxHashMap, FxHasher};

use crate::utils::ServerEntryModules;

pub fn track_server_component_changes(
  compilation: &Compilation,
  prev_server_component_hashs: &mut IdentifierMap<u64>,
) -> FxHashMap<String, IdentifierSet> {
  let module_graph = compilation.get_module_graph();
  let server_entry_modules = ServerEntryModules::new(compilation, &module_graph);

  let mut visited_modules: IdentifierSet = Default::default();
  let mut changed_server_components_per_entry: FxHashMap<String, IdentifierSet> =
    Default::default();
  let mut cur_server_component_hashs = Default::default();

  for (server_entry_module, entry_name, runtime) in server_entry_modules {
    visited_modules.clear();

    let mut changed_server_components = changed_server_components_per_entry
      .entry(entry_name.to_string())
      .or_insert_with(IdentifierSet::default);

    traverse_server_components(
      compilation,
      &module_graph,
      server_entry_module,
      &runtime,
      prev_server_component_hashs,
      &mut visited_modules,
      &mut cur_server_component_hashs,
      &mut changed_server_components,
    )
  }

  *prev_server_component_hashs = cur_server_component_hashs;

  changed_server_components_per_entry
}

fn traverse_server_components(
  compilation: &Compilation,
  module_graph: &ModuleGraphRef<'_>,
  module: &dyn Module,
  runtime: &RuntimeSpec,
  prev_server_component_hashs: &IdentifierMap<u64>,
  visited_modules: &mut IdentifierSet,
  cur_server_component_hashs: &mut IdentifierMap<u64>,
  changed_server_components: &mut IdentifierSet,
) {
  if let Some(rsc) = module.build_info().rsc.as_ref()
    && rsc.module_type == RSCModuleType::Client
  {
    return;
  }

  let module_identifier = module.identifier();
  if visited_modules.contains(&module_identifier) {
    return;
  }
  visited_modules.insert(module_identifier);

  let Some(module) = compilation.module_by_identifier(&module_identifier) else {
    return;
  };
  let Some(source) = module.source() else {
    return;
  };
  let mut hasher = FxHasher::default();
  source.hash(&mut hasher);
  let cur_hash = hasher.finish();
  if prev_server_component_hashs
    .get(&module_identifier)
    .is_some_and(|prev| *prev != cur_hash)
  {
    changed_server_components.insert(module_identifier);
  }
  cur_server_component_hashs.insert(module_identifier, cur_hash);

  for dependency_id in module_graph.get_outgoing_deps_in_order(&module.identifier()) {
    let Some(resolved_module) = module_graph
      .connection_by_dependency_id(dependency_id)
      .and_then(|c| module_graph.module_by_identifier(&c.resolved_module))
    else {
      continue;
    };

    traverse_server_components(
      compilation,
      module_graph,
      resolved_module.as_ref(),
      runtime,
      prev_server_component_hashs,
      visited_modules,
      cur_server_component_hashs,
      changed_server_components,
    );
  }
}
