use std::hash::{Hash, Hasher};

use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::{Compilation, Module, ModuleGraphRef, RscModuleType};
use rustc_hash::{FxHashMap, FxHasher};

use crate::constants::LAYERS_NAMES;

pub fn track_server_component_changes(
  compilation: &Compilation,
  prev_server_component_hashes: &mut IdentifierMap<u64>,
) -> FxHashMap<String, IdentifierSet> {
  let module_graph = compilation.get_module_graph();

  let mut visited_modules: IdentifierSet = Default::default();
  let mut changed_server_components_per_entry: FxHashMap<String, IdentifierSet> =
    Default::default();
  let mut cur_server_component_hashes = Default::default();

  for (entry_name, entry_data) in &compilation.entries {
    visited_modules.clear();

    let changed_server_components = changed_server_components_per_entry
      .entry(entry_name.to_string())
      .or_default();

    let entry_dependency_id = entry_data.dependencies[0];
    let Some(resolved_module) = module_graph
      .get_resolved_module(&entry_dependency_id)
      .and_then(|identifier| compilation.module_by_identifier(identifier))
    else {
      continue;
    };

    collect_changed_server_components(
      compilation,
      &module_graph,
      resolved_module.as_ref(),
      prev_server_component_hashes,
      &mut visited_modules,
      &mut cur_server_component_hashes,
      changed_server_components,
    )
  }

  *prev_server_component_hashes = cur_server_component_hashes;
  changed_server_components_per_entry
}

#[allow(clippy::too_many_arguments)]
fn collect_changed_server_components(
  compilation: &Compilation,
  module_graph: &ModuleGraphRef<'_>,
  module: &dyn Module,
  prev_server_component_hashes: &IdentifierMap<u64>,
  visited_modules: &mut IdentifierSet,
  cur_server_component_hashes: &mut IdentifierMap<u64>,
  changed_server_components: &mut IdentifierSet,
) {
  let module_identifier = module.identifier();
  if visited_modules.contains(&module_identifier) {
    return;
  }
  visited_modules.insert(module_identifier);

  if let Some(rsc) = module.build_info().rsc.as_ref()
    && rsc.module_type.contains(RscModuleType::Client)
  {
    return;
  }

  if module
    .get_layer()
    .is_some_and(|layer| layer == LAYERS_NAMES.react_server_components)
  {
    let Some(module) = compilation.module_by_identifier(&module_identifier) else {
      return;
    };
    let Some(source) = module.source() else {
      return;
    };
    let mut hasher = FxHasher::default();
    source.hash(&mut hasher);
    let cur_hash = hasher.finish();
    if prev_server_component_hashes
      .get(&module_identifier)
      .is_some_and(|prev| *prev != cur_hash)
    {
      changed_server_components.insert(module_identifier);
    }
    cur_server_component_hashes.insert(module_identifier, cur_hash);
  }

  for dependency_id in module_graph.get_outgoing_deps_in_order(&module_identifier) {
    let Some(resolved_module) = module_graph
      .connection_by_dependency_id(dependency_id)
      .and_then(|c| module_graph.module_by_identifier(&c.resolved_module))
    else {
      continue;
    };

    collect_changed_server_components(
      compilation,
      module_graph,
      resolved_module.as_ref(),
      prev_server_component_hashes,
      visited_modules,
      cur_server_component_hashes,
      changed_server_components,
    );
  }
}
