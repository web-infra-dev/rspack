// Port of webpack's HMR sibling injection and retained-module updates.
use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::{
  ChunkGraph, Compilation, CompilationRecords, ModuleIdMap, ModuleIdentifier, RuntimeSpec,
  chunk_graph_chunk::{ChunkId, ChunkIdMap},
};
use rspack_hash::RspackHashDigest;

#[derive(Default)]
pub(crate) struct ModuleUpdates {
  pub updated: IdentifierSet,
  pub supplied: IdentifierSet,
}

// A shared chunk may have generated the factory only for a merged runtime.
// Render the same factory in the receiving chunk's update.
fn ensure_runtime(
  compilation: &mut Compilation,
  module: ModuleIdentifier,
  runtime: &RuntimeSpec,
  source_runtime: &RuntimeSpec,
) -> bool {
  if compilation
    .code_generation_results
    .try_get(&module, Some(runtime))
    .is_ok()
  {
    return true;
  }
  let Ok(result) = compilation
    .code_generation_results
    .try_get(&module, Some(source_runtime))
  else {
    return false;
  };
  let result = (**result).clone();
  compilation
    .code_generation_results
    .insert(module, result, [runtime.clone()]);
  let requirements = compilation
    .cgm_runtime_requirements_artifact
    .get(&module, source_runtime)
    .copied();
  if let Some(requirements) = requirements {
    let mut map = compilation
      .cgm_runtime_requirements_artifact
      .remove(&module)
      .unwrap_or_default();
    map.set(runtime.clone(), requirements);
    compilation
      .cgm_runtime_requirements_artifact
      .set_runtime_requirements(module, map);
  }
  true
}

fn record_hash(
  compilation: &mut Compilation,
  records: &CompilationRecords,
  module: ModuleIdentifier,
  carrier: &ChunkId,
  runtime: &RuntimeSpec,
) -> bool {
  let id = ChunkGraph::get_module_id(&compilation.module_ids_artifact, module)
    .expect("module id")
    .clone();
  let hash = compilation
    .code_generation_results
    .get_hash(&module, Some(runtime))
    .expect("module hash")
    .clone();
  let changed = records
    .modules
    .get(&id)
    .and_then(|hashes| hashes.get(carrier))
    != Some(&hash);
  compilation
    .hot_module_hashes
    .entry(id)
    .or_default()
    .insert(carrier.clone(), hash);
  changed
}

fn unchanged(
  compilation: &Compilation,
  records: &CompilationRecords,
  module: ModuleIdentifier,
  source: &ChunkId,
  hash: &RspackHashDigest,
  runtime: &RuntimeSpec,
) -> bool {
  let id = ChunkGraph::get_module_id(&compilation.module_ids_artifact, module).expect("module id");
  let Some(hashes) = records.modules.get(id) else {
    return false;
  };
  if hashes.get(source) == Some(hash) {
    return true;
  }
  runtime.iter().all(|runtime| {
    hashes.iter().any(|(chunk, old_hash)| {
      old_hash == hash
        && records
          .chunks
          .get(chunk)
          .is_some_and(|(runtimes, _)| runtimes.contains(runtime))
    })
  })
}

pub(crate) fn collect_module_updates(
  compilation: &mut Compilation,
  records: &CompilationRecords,
) -> ChunkIdMap<ModuleUpdates> {
  let graph = &compilation.build_chunk_graph_artifact;
  let current_chunks: ChunkIdMap<_> = graph
    .chunk_by_ukey
    .values()
    .map(|chunk| (chunk.expect_id().clone(), chunk.ukey()))
    .collect();
  let current_siblings = CompilationRecords::record_chunk_siblings(compilation);
  let modules_by_id = compilation
    .module_ids_artifact
    .iter()
    .map(|(module, id)| (id.clone(), *module))
    .collect::<ModuleIdMap<_>>();
  let mut updates = ChunkIdMap::default();

  for (id, (_, old_modules)) in &records.chunks {
    let Some(&key) = current_chunks.get(id) else {
      continue;
    };
    let graph = &compilation.build_chunk_graph_artifact;
    let chunk = graph.chunk_by_ukey.expect_get(&key);
    let runtime = chunk.runtime().clone();
    let recipients: RuntimeSpec = runtime.intersection(&records.runtimes).copied().collect();
    if recipients.is_empty() {
      continue;
    }
    let actual = graph.chunk_graph.get_chunk_modules_identifier(&key).clone();
    let mut update = ModuleUpdates::default();
    for &module in &actual {
      let Some(module_id) = ChunkGraph::get_module_id(&compilation.module_ids_artifact, module)
      else {
        continue;
      };
      if records
        .modules
        .get(module_id)
        .and_then(|hashes| hashes.get(id))
        != compilation
          .code_generation_results
          .get_hash(&module, Some(&runtime))
      {
        update.updated.insert(module);
      }
    }

    let mut retained = IdentifierSet::default();
    let mut queue = old_modules
      .iter()
      .filter_map(|id| modules_by_id.get(id).copied())
      .collect::<Vec<_>>();
    let mut cursor = 0;
    while cursor < queue.len() {
      let module = queue[cursor];
      cursor += 1;
      if actual.contains(&module) || retained.contains(&module) {
        continue;
      }
      let graph = &compilation.build_chunk_graph_artifact;
      let owning_chunks = graph.chunk_graph.get_module_chunks(module);
      // Existing siblings already have their own update stream.
      if owning_chunks.iter().any(|key| {
        let owner = graph.chunk_by_ukey.expect_get(key).expect_id();
        current_siblings[id].contains(owner) && records.chunks.contains_key(owner)
      }) {
        continue;
      }
      let source = owning_chunks.iter().find_map(|key| {
        let owner = graph.chunk_by_ukey.expect_get(key);
        (!owner.runtime().is_disjoint(&runtime)).then(|| owner.runtime().clone())
      });
      let Some(source) = source else { continue };
      if !ensure_runtime(compilation, module, &runtime, &source) {
        continue;
      }
      retained.insert(module);
      if record_hash(compilation, records, module, id, &runtime) {
        update.updated.insert(module);
      }
      let mg = compilation.get_module_graph();
      if let Some(module) = mg.module_by_identifier(&module) {
        for dependency in module.get_dependencies() {
          if let Some(connection) = mg.connection_by_dependency_id(dependency.id())
            && connection.is_target_active(
              mg,
              Some(&runtime),
              &compilation.module_graph_cache_artifact,
              &compilation
                .build_module_graph_artifact
                .side_effects_state_artifact,
              &compilation.exports_info_artifact,
            )
          {
            queue.push(*connection.module_identifier());
          }
        }
      }
    }

    // Keep the carrier's future updates responsible for these factories too.
    let mut injected = IdentifierMap::default();
    for sibling in &current_siblings[id] {
      if records
        .chunk_siblings
        .get(id)
        .is_some_and(|siblings| siblings.contains(sibling))
      {
        continue;
      }
      let graph = &compilation.build_chunk_graph_artifact;
      let sibling = graph.chunk_by_ukey.expect_get(&current_chunks[sibling]);
      if sibling.runtime().is_disjoint(&recipients) {
        continue;
      }
      for &module in graph
        .chunk_graph
        .get_chunk_modules_identifier(&sibling.ukey())
      {
        if !actual.contains(&module) && !retained.contains(&module) {
          injected.entry(module).or_insert((
            sibling.expect_id().clone(),
            sibling.runtime().clone(),
            true,
          ));
        }
      }
    }
    if let Some(previous) = records.injected_modules.get(id) {
      for module in previous
        .iter()
        .filter_map(|id| modules_by_id.get(id).copied())
      {
        if actual.contains(&module) || injected.contains_key(&module) {
          continue;
        }
        let graph = &compilation.build_chunk_graph_artifact;
        if let Some(owner) = graph
          .chunk_graph
          .get_module_chunks(module)
          .iter()
          .map(|key| graph.chunk_by_ukey.expect_get(key))
          .find(|chunk| !chunk.runtime().is_disjoint(&recipients))
        {
          injected.insert(
            module,
            (owner.expect_id().clone(), owner.runtime().clone(), false),
          );
        }
      }
    }
    for (module, (source, source_runtime, supply)) in injected {
      if !ensure_runtime(compilation, module, &runtime, &source_runtime) {
        continue;
      }
      let module_id = ChunkGraph::get_module_id(&compilation.module_ids_artifact, module)
        .expect("module id")
        .clone();
      compilation
        .hot_injected_modules
        .entry(id.clone())
        .or_default()
        .insert(module_id);
      let changed = record_hash(compilation, records, module, id, &runtime);
      if !supply && !changed {
        continue;
      }
      if supply
        && unchanged(
          compilation,
          records,
          module,
          &source,
          compilation
            .code_generation_results
            .get_hash(&module, Some(&source_runtime))
            .expect("module hash"),
          &recipients,
        )
      {
        update.supplied.insert(module);
      }
      update.updated.insert(module);
    }
    updates.insert(id.clone(), update);
  }
  updates
}
