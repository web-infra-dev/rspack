//! Performance-optimized scope hoisting group computation.
//!
//! Replaces the recursive `try_to_add` algorithm with a compact-indexed BFS + fixed-point
//! removal approach.
//!
//! Key optimizations:
//! - Compact u32-indexed adjacency lists (O(1) array access vs hash map lookup)
//! - Pre-validated inner module set (single global pass)
//! - Eager static constraint checking during BFS (Phase A) to minimize Phase B work
//! - Vec<bool> bitset for visited/group membership
//! - No recursion, no snapshot/rollback
//! - Reusable buffers across root iterations

use std::{collections::VecDeque, rc::Rc};

use rspack_collections::{IdentifierIndexSet, IdentifierMap, IdentifierSet, UkeySet};
use rspack_core::{
  ChunkByUkey, ChunkGraph, ChunkUkey, Compilation, ExportsInfoArtifact, ModuleGraph,
  ModuleGraphCacheArtifact, ModuleIdentifier, PrefetchExportsInfoMode, RuntimeCondition,
  RuntimeSpec, filter_runtime,
};
use rustc_hash::FxHashMap as HashMap;

use super::module_concatenation_plugin::{
  ConcatConfiguration, ModuleConcatenationPlugin, NoRuntimeModuleCache, RuntimeIdentifierCache,
  is_connection_active_in_runtime,
};

type Idx = u32;

/// Compact indexed representation of the ESM module graph.
/// All lookups are O(1) array accesses.
struct CompactGraph {
  len: usize,
  /// Index -> ModuleIdentifier
  ids: Vec<ModuleIdentifier>,
  /// ModuleIdentifier -> Index
  idx_of: IdentifierMap<Idx>,
  /// ESM incoming edges (importers): module_idx -> [source_idx, ...]
  esm_in: Vec<Vec<Idx>>,
  /// Pre-validated: can this module be an inner module?
  /// Only checks possible_inners membership (runtime-dependent checks are in Phase A/B)
  valid_inner: Vec<bool>,
}

impl CompactGraph {
  fn build(
    possible_inners: &IdentifierSet,
    relevant_modules: &[ModuleIdentifier],
    module_cache: &HashMap<ModuleIdentifier, NoRuntimeModuleCache>,
  ) -> Self {
    // Collect all modules that participate in scope hoisting
    let all_modules: Vec<ModuleIdentifier> = possible_inners
      .iter()
      .chain(relevant_modules.iter())
      .copied()
      .collect::<IdentifierSet>()
      .into_iter()
      .collect();

    let len = all_modules.len();
    let mut idx_of = IdentifierMap::default();
    idx_of.reserve(len);
    for (i, id) in all_modules.iter().enumerate() {
      idx_of.insert(*id, i as Idx);
    }

    let mut esm_in = vec![Vec::new(); len];

    for (i, module_id) in all_modules.iter().enumerate() {
      if let Some(cached) = module_cache.get(module_id) {
        // Build incoming ESM edges (importers)
        for (origin_module, connections) in cached.incomings.iter() {
          if let Some(origin) = origin_module {
            let has_esm = connections.iter().any(|c| c.is_esm);
            if has_esm {
              if let Some(&origin_idx) = idx_of.get(origin) {
                esm_in[i].push(origin_idx);
              }
            }
          }
        }
      }
    }

    // Pre-validate inner modules (only runtime-independent check)
    let mut valid_inner = vec![false; len];
    for (i, module_id) in all_modules.iter().enumerate() {
      valid_inner[i] = possible_inners.contains(module_id);
    }

    CompactGraph {
      len,
      ids: all_modules,
      idx_of,
      esm_in,
      valid_inner,
    }
  }

  #[inline]
  fn idx(&self, id: &ModuleIdentifier) -> Option<Idx> {
    self.idx_of.get(id).copied()
  }
}

/// Check all static (group-independent) constraints for a candidate inner module.
/// Returns true if the module passes all checks and can potentially be an inner module.
///
/// Static checks (matching try_to_add order):
/// 1. possible_inners membership (already done via valid_inner in caller)
/// 2. chunk containment (already done in caller)
/// 3. Non-module references (active incoming from None origin)
/// 4. Importer not in root's chunks
/// 5. Non-ESM connections from active importers
/// 6. Runtime-conditional importers (multi-runtime)
#[inline]
fn passes_static_checks(
  cached: &NoRuntimeModuleCache,
  runtime: &RuntimeSpec,
  root_chunks: &UkeySet<ChunkUkey>,
  chunk_graph: &ChunkGraph,
  chunk_by_ukey: &ChunkByUkey,
  module_graph: &ModuleGraph,
  module_graph_cache: &ModuleGraphCacheArtifact,
  exports_info_artifact: &ExportsInfoArtifact,
  module_cache: &HashMap<ModuleIdentifier, NoRuntimeModuleCache>,
) -> bool {
  // Check 3: Non-module references
  if let Some(non_module_connections) = cached.incomings.get(&None) {
    let has_active = non_module_connections.iter().any(|connection| {
      is_connection_active_in_runtime(
        connection,
        Some(runtime),
        &cached.runtime,
        module_graph,
        module_graph_cache,
        exports_info_artifact,
      )
    });
    if has_active {
      return false;
    }
  }

  // Process incoming connections from modules
  for (origin_module, connections) in cached.incomings.iter() {
    let origin = match origin_module {
      Some(o) => o,
      None => continue, // already handled above
    };

    // Skip orphan modules
    let origin_num_chunks = module_cache.get(origin).map_or_else(
      || chunk_graph.get_number_of_module_chunks(*origin),
      |m| m.number_of_chunks,
    );
    if origin_num_chunks == 0 {
      continue;
    }

    // Check runtime intersection
    let is_intersect = if let Some(origin_cache) = module_cache.get(origin) {
      !runtime.is_disjoint(&origin_cache.runtime)
    } else {
      let mut origin_runtime = RuntimeSpec::default();
      for r in chunk_graph.get_module_runtimes_iter(*origin, chunk_by_ukey) {
        origin_runtime.extend(r);
      }
      !runtime.is_disjoint(&origin_runtime)
    };
    if !is_intersect {
      continue;
    }

    // Filter to active connections
    let active_connections: Vec<_> = connections
      .iter()
      .filter(|connection| {
        is_connection_active_in_runtime(
          connection,
          Some(runtime),
          &cached.runtime,
          module_graph,
          module_graph_cache,
          exports_info_artifact,
        )
      })
      .collect();
    if active_connections.is_empty() {
      continue;
    }

    // Check 4: Importer must be in root's chunks
    let importer_in_root_chunks = root_chunks
      .iter()
      .all(|chunk| chunk_graph.is_module_in_chunk(origin, *chunk));
    if !importer_in_root_chunks {
      return false;
    }

    // Check 5: No non-ESM connections (pre-computed)
    let has_non_esm = active_connections
      .iter()
      .any(|connection| !connection.is_esm);
    if has_non_esm {
      return false;
    }
  }

  // Check 6: Runtime-conditional importers (multi-runtime only)
  if runtime.len() > 1 {
    if has_runtime_conditional_importers(
      runtime,
      cached,
      module_graph,
      module_graph_cache,
      exports_info_artifact,
    ) {
      return false;
    }
  }

  true
}

/// Main entry point: compute concatenation groups using the fast algorithm.
///
/// Returns (groups, used_as_inner, stats_size_sum, stats_empty_configurations)
pub fn find_concat_groups_fast(
  compilation: &Compilation,
  relevant_modules: &[ModuleIdentifier],
  possible_inners: &IdentifierSet,
  module_cache: &HashMap<ModuleIdentifier, NoRuntimeModuleCache>,
) -> (Vec<ConcatConfiguration>, IdentifierSet, usize, usize) {
  let graph = CompactGraph::build(possible_inners, relevant_modules, module_cache);

  let chunk_graph = &compilation.build_chunk_graph_artifact.chunk_graph;
  let chunk_by_ukey = &compilation.build_chunk_graph_artifact.chunk_by_ukey;
  let module_graph_cache = &compilation.module_graph_cache_artifact;

  let mut concat_configurations: Vec<ConcatConfiguration> = Vec::new();
  let mut used_as_inner = IdentifierSet::default();
  let mut stats_size_sum = 0usize;
  let mut stats_empty_configurations = 0usize;

  let mut imports_cache = RuntimeIdentifierCache::<Rc<IdentifierIndexSet>>::default();

  // Reusable buffers (avoid allocation per root)
  let mut in_group = vec![false; graph.len]; // bitset: is module in the tentative group?
  let mut bfs_visited = vec![false; graph.len]; // bitset: visited during BFS?
  let mut group_indices: Vec<Idx> = Vec::new(); // indices of modules in the group
  let mut all_visited: Vec<Idx> = Vec::new(); // ALL visited nodes (for cleanup)
  let mut bfs_queue: VecDeque<Idx> = VecDeque::new();
  let mut to_remove: Vec<Idx> = Vec::new();

  for current_root in relevant_modules.iter() {
    if used_as_inner.contains(current_root) {
      continue;
    }

    let root_idx = match graph.idx(current_root) {
      Some(idx) => idx,
      None => continue,
    };

    let NoRuntimeModuleCache { runtime, .. } = match module_cache.get(current_root) {
      Some(c) => c,
      None => continue,
    };

    let module_graph = compilation.get_module_graph();
    let exports_info = compilation
      .exports_info_artifact
      .get_prefetched_exports_info(current_root, PrefetchExportsInfoMode::Default);
    let filtered_runtime = filter_runtime(Some(runtime), |r| exports_info.is_module_used(r));
    let active_runtime = match filtered_runtime {
      RuntimeCondition::Boolean(true) => Some(runtime.clone()),
      RuntimeCondition::Boolean(false) => None,
      RuntimeCondition::Spec(spec) => Some(spec),
    };

    let root_chunks = chunk_graph.get_module_chunks(*current_root);

    // ── Phase A: BFS expand with eager static checks ─────────────────────
    // Seed with root's ESM imports (using get_imports with active_runtime, matching old algorithm)
    let imports = ModuleConcatenationPlugin::get_imports(
      module_graph,
      module_graph_cache,
      &compilation.exports_info_artifact,
      *current_root,
      active_runtime.as_ref(),
      &mut imports_cache,
      module_cache,
    );

    bfs_queue.clear();

    in_group[root_idx as usize] = true;
    bfs_visited[root_idx as usize] = true;
    group_indices.push(root_idx);

    for imp in imports.iter() {
      if let Some(idx) = graph.idx(imp) {
        if !bfs_visited[idx as usize] {
          bfs_queue.push_back(idx);
          bfs_visited[idx as usize] = true;
          all_visited.push(idx);
        }
      }
    }

    while let Some(candidate) = bfs_queue.pop_front() {
      let cu = candidate as usize;

      // Check 1: possible_inners membership (O(1))
      if !graph.valid_inner[cu] {
        continue;
      }

      // Check 2: Chunk containment — candidate must be in ALL root's chunks
      let candidate_id = &graph.ids[cu];
      let all_chunks = root_chunks
        .iter()
        .all(|chunk| chunk_graph.is_module_in_chunk(candidate_id, *chunk));
      if !all_chunks {
        continue;
      }

      // Checks 3-6: All static incoming-connection constraints
      let cached = match module_cache.get(candidate_id) {
        Some(c) => c,
        None => continue,
      };
      if !passes_static_checks(
        cached,
        runtime,
        root_chunks,
        chunk_graph,
        chunk_by_ukey,
        module_graph,
        module_graph_cache,
        &compilation.exports_info_artifact,
        module_cache,
      ) {
        continue;
      }

      in_group[cu] = true;
      group_indices.push(candidate);

      // Expand: add candidate's ESM imports to BFS
      let candidate_imports = ModuleConcatenationPlugin::get_imports(
        module_graph,
        module_graph_cache,
        &compilation.exports_info_artifact,
        graph.ids[cu],
        Some(runtime),
        &mut imports_cache,
        module_cache,
      );
      for imp in candidate_imports.iter() {
        if let Some(idx) = graph.idx(imp) {
          if !bfs_visited[idx as usize] {
            bfs_visited[idx as usize] = true;
            all_visited.push(idx);
            bfs_queue.push_back(idx);
          }
        }
      }

      // Expand: add candidate's ESM importers to BFS (pull in required importers)
      for &source in &graph.esm_in[cu] {
        if !bfs_visited[source as usize] {
          bfs_visited[source as usize] = true;
          all_visited.push(source);
          bfs_queue.push_back(source);
        }
      }
    }

    // ── Phase B: Fixed-point removal (group-dependent check only) ────────
    // Only check: is each module's active importer in the group?
    let module_graph = compilation.get_module_graph();
    loop {
      to_remove.clear();

      for &module_idx in &group_indices {
        let mu = module_idx as usize;
        if !in_group[mu] {
          continue; // Already removed in a previous iteration
        }
        if module_idx == root_idx {
          continue; // Never remove the root
        }

        let module_id = &graph.ids[mu];
        let cached = match module_cache.get(module_id) {
          Some(c) => c,
          None => continue,
        };

        let mut should_remove = false;

        for (origin_module, connections) in cached.incomings.iter() {
          let origin = match origin_module {
            Some(o) => o,
            None => continue, // Non-module refs already checked in Phase A
          };

          // Skip orphan modules
          let origin_num_chunks = module_cache.get(origin).map_or_else(
            || chunk_graph.get_number_of_module_chunks(*origin),
            |m| m.number_of_chunks,
          );
          if origin_num_chunks == 0 {
            continue;
          }

          // Check runtime intersection
          let is_intersect = if let Some(origin_cache) = module_cache.get(origin) {
            !runtime.is_disjoint(&origin_cache.runtime)
          } else {
            let mut origin_runtime = RuntimeSpec::default();
            for r in chunk_graph.get_module_runtimes_iter(*origin, chunk_by_ukey) {
              origin_runtime.extend(r);
            }
            !runtime.is_disjoint(&origin_runtime)
          };
          if !is_intersect {
            continue;
          }

          // Filter to active connections
          let has_active = connections.iter().any(|connection| {
            is_connection_active_in_runtime(
              connection,
              Some(runtime),
              &cached.runtime,
              module_graph,
              module_graph_cache,
              &compilation.exports_info_artifact,
            )
          });
          if !has_active {
            continue;
          }

          // The importer is active — check if it's in the group
          if let Some(&origin_idx) = graph.idx_of.get(origin) {
            if !in_group[origin_idx as usize] {
              should_remove = true;
              break;
            }
          } else {
            should_remove = true;
            break;
          }
        }

        if should_remove {
          to_remove.push(module_idx);
        }
      }

      if to_remove.is_empty() {
        break; // Stable
      }

      for &idx in &to_remove {
        in_group[idx as usize] = false;
      }
    }

    // ── Phase C: Build ConcatConfiguration ───────────────────────────────
    let surviving_count = group_indices
      .iter()
      .filter(|&&idx| in_group[idx as usize])
      .count();

    if surviving_count > 1 {
      let mut config = ConcatConfiguration::new(*current_root, active_runtime);
      for &idx in &group_indices {
        if in_group[idx as usize] && idx != root_idx {
          let mid = graph.ids[idx as usize];
          config.add(mid);
          used_as_inner.insert(mid);
        }
      }
      stats_size_sum += config.get_modules().len();
      concat_configurations.push(config);
    } else {
      stats_empty_configurations += 1;
    }

    // Clean up reusable buffers for next iteration.
    // IMPORTANT: Reset ALL visited nodes (not just group members) to avoid
    // contaminating future roots' BFS. Modules that failed validation in one
    // root's BFS must be re-discoverable by subsequent roots.
    for &idx in &all_visited {
      bfs_visited[idx as usize] = false;
    }
    for &idx in &group_indices {
      in_group[idx as usize] = false;
    }
    in_group[root_idx as usize] = false;
    bfs_visited[root_idx as usize] = false;
    group_indices.clear();
    all_visited.clear();
  }

  (
    concat_configurations,
    used_as_inner,
    stats_size_sum,
    stats_empty_configurations,
  )
}

/// Check if a module has runtime-conditional importers (multi-runtime scenario).
/// Equivalent to lines 525-595 of the original try_to_add.
fn has_runtime_conditional_importers(
  runtime: &RuntimeSpec,
  cached: &NoRuntimeModuleCache,
  module_graph: &ModuleGraph,
  module_graph_cache: &ModuleGraphCacheArtifact,
  exports_info_artifact: &ExportsInfoArtifact,
) -> bool {
  for (origin_module, connections) in cached.incomings.iter() {
    if origin_module.is_none() {
      continue;
    }

    let mut current_runtime_condition = RuntimeCondition::Boolean(false);
    let mut all_true = false;

    for cached_incoming in connections.iter() {
      let connection = module_graph
        .connection_by_dependency_id(&cached_incoming.dependency_id)
        .expect("should have connection");
      let runtime_condition = filter_runtime(Some(runtime), |rt| {
        connection.is_target_active(module_graph, rt, module_graph_cache, exports_info_artifact)
      });

      if runtime_condition == RuntimeCondition::Boolean(false) {
        continue;
      }
      if runtime_condition == RuntimeCondition::Boolean(true) {
        all_true = true;
        break;
      }

      // RuntimeCondition::Spec
      if current_runtime_condition != RuntimeCondition::Boolean(false) {
        current_runtime_condition
          .as_spec_mut()
          .expect("should be spec")
          .extend(runtime_condition.as_spec().expect("should be spec"));
      } else {
        current_runtime_condition = runtime_condition;
      }
    }

    if !all_true && current_runtime_condition != RuntimeCondition::Boolean(false) {
      return true;
    }
  }

  false
}
