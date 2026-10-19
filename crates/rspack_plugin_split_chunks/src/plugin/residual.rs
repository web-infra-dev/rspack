//! Incremental updates of native candidates after committed source-edge removals.

use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::{ChunkGroupUkey, ChunkUkey, Compilation, ModuleIdentifier};
use rustc_hash::{FxHashMap, FxHashSet};

use super::{ModuleGroupMap, module_group::get_key};
use crate::{
  CacheGroup, SplitChunkSizes, SplitChunksPlugin,
  common::{ModuleChunkMap, ModuleSizes},
  module_group::{ModuleGroup, ModuleGroupKey},
};

struct Candidate {
  key: ModuleGroupKey,
  modules: IdentifierSet,
  chunks: FxHashSet<ChunkUkey>,
  named_sources: Option<IdentifierMap<FxHashSet<ChunkUkey>>>,
  changed: bool,
  sizes: SplitChunkSizes,
}

/// One owned snapshot of each prepared candidate survives native greedy cleanup.
/// Committed removals touch only indexed members and shrink one residual state.
/// Equal keys merge immediately, without branching or rediscovering support.
#[derive(Default)]
pub(super) struct ResidualCandidates {
  groups: Vec<Option<Candidate>>,
  keys: FxHashMap<ModuleGroupKey, usize>,
  members: IdentifierMap<FxHashSet<usize>>,
  affected: usize,
  shrunk: usize,
}

impl ResidualCandidates {
  pub(super) fn capture(
    groups: &mut ModuleGroupMap,
    cache_groups: &[CacheGroup],
    sizes: &ModuleSizes,
  ) -> Self {
    let mut state = Self::default();
    for (key, group) in groups {
      let cache = group.get_cache_group(cache_groups);
      // Shrinking cannot make an undersized source type large enough. Retain
      // whole mixed-type candidates when any type can fit: later removals may
      // eliminate a different, violating type. Do not aggregate undersized
      // native candidates into newly eligible supplementary groups.
      if !Self::can_fit(group.get_sizes(sizes), cache) {
        continue;
      }
      let named_sources = group.chunk_name.as_ref().map(|_| {
        group
          .modules
          .iter()
          .map(|module| {
            (
              *module,
              group
                .get_module_chunks(module)
                .expect("selected sources")
                .clone(),
            )
          })
          .collect()
      });
      let candidate_sizes = group.get_sizes(sizes).clone();
      state.insert(
        Candidate {
          key: key.clone(),
          modules: group.modules.clone(),
          chunks: group
            .shared_module_chunks()
            .unwrap_or(&group.chunks)
            .clone(),
          named_sources,
          changed: false,
          sizes: candidate_sizes,
        },
        sizes,
      );
    }
    state
  }

  pub(super) fn len(&self) -> usize {
    self.keys.len()
  }

  fn can_fit(sizes: &SplitChunkSizes, cache: &CacheGroup) -> bool {
    sizes
      .iter()
      .any(|(ty, size)| *size > 0.0 && *size >= cache.min_size.get(ty).copied().unwrap_or_default())
  }

  fn discard(&mut self, index: usize) {
    let candidate = self.groups[index].take().expect("live candidate");
    self.keys.remove(&candidate.key);
    for module in candidate.modules {
      self
        .members
        .get_mut(&module)
        .expect("member index")
        .remove(&index);
    }
  }

  fn insert(&mut self, candidate: Candidate, sizes: &ModuleSizes) {
    if let Some(&index) = self.keys.get(&candidate.key) {
      let existing = self.groups[index].as_mut().expect("live candidate");
      debug_assert!(existing.named_sources.is_none() && candidate.named_sources.is_none());
      existing.changed |= candidate.changed;
      for module in candidate.modules {
        if existing.modules.insert(module) {
          for (ty, size) in sizes.get(&module).expect("module sizes").iter() {
            *existing.sizes.entry(*ty).or_default() += size;
          }
          self.members.entry(module).or_default().insert(index);
        }
      }
    } else {
      let index = self.groups.len();
      for module in &candidate.modules {
        self.members.entry(*module).or_default().insert(index);
      }
      self.keys.insert(candidate.key.clone(), index);
      self.groups.push(Some(candidate));
    }
  }

  pub(super) fn remove(
    &mut self,
    placed: &ModuleChunkMap,
    cache_groups: &[CacheGroup],
    indices: &FxHashMap<ChunkUkey, u32>,
    sizes: &ModuleSizes,
  ) {
    match placed {
      ModuleChunkMap::Shared { modules, chunks } => {
        self.remove_shared(
          modules.iter().copied(),
          chunks,
          cache_groups,
          indices,
          sizes,
        );
      }
      ModuleChunkMap::ByModule(modules) => {
        // Keep updates deterministic when different modules lose different edges.
        let mut modules = modules.iter().collect::<Vec<_>>();
        modules.sort_unstable_by_key(|(module, _)| **module);
        for (module, chunks) in modules {
          self.remove_shared(
            std::iter::once(*module),
            chunks,
            cache_groups,
            indices,
            sizes,
          );
        }
      }
    }
  }

  fn remove_shared(
    &mut self,
    modules: impl Iterator<Item = ModuleIdentifier>,
    removed: &FxHashSet<ChunkUkey>,
    cache_groups: &[CacheGroup],
    indices: &FxHashMap<ChunkUkey, u32>,
    sizes: &ModuleSizes,
  ) {
    let mut affected = FxHashMap::<usize, Vec<ModuleIdentifier>>::default();
    for module in modules {
      if let Some(groups) = self.members.get(&module) {
        for index in groups {
          affected.entry(*index).or_default().push(module);
        }
      }
    }
    for (index, modules) in affected {
      let group = self.groups[index].as_mut().expect("indexed candidate");
      if group.chunks.is_disjoint(removed) {
        continue;
      }
      self.affected += 1;
      let cache = match &group.key {
        ModuleGroupKey::Named {
          cache_group_index, ..
        }
        | ModuleGroupKey::Anonymous {
          cache_group_index, ..
        } => *cache_group_index,
      };
      let minimum = (cache_groups[cache as usize].min_chunks as usize).max(1);
      if let Some(sources) = &mut group.named_sources {
        for module in modules {
          let chunks = sources.get_mut(&module).expect("named member");
          let previous_len = chunks.len();
          chunks.retain(|chunk| !removed.contains(chunk));
          group.changed |= chunks.len() != previous_len;
          if chunks.len() < minimum {
            sources.remove(&module);
            group.modules.remove(&module);
            for (ty, size) in sizes.get(&module).expect("module sizes").iter() {
              let total = group.sizes.get_mut(ty).expect("source type");
              *total = (*total - size).max(0.0);
            }
            self
              .members
              .get_mut(&module)
              .expect("member index")
              .remove(&index);
          }
        }
        // The union may keep obsolete chunks, which only adds conservative wakeups.
        if group.modules.is_empty() || !Self::can_fit(&group.sizes, &cache_groups[cache as usize]) {
          self.discard(index);
        }
        continue;
      }
      let remaining_len = group.chunks.len() - group.chunks.intersection(removed).count();
      group.changed = true;
      if remaining_len < minimum {
        // These members cannot form an eligible residual on this source set.
        // Keep the untouched members, including ones pruned by source type in
        // the native queue, rather than creating an empty residual branch.
        for module in modules {
          group.modules.remove(&module);
          for (ty, size) in sizes.get(&module).expect("module sizes").iter() {
            let total = group.sizes.get_mut(ty).expect("source type");
            *total = (*total - size).max(0.0);
          }
          self
            .members
            .get_mut(&module)
            .expect("member index")
            .remove(&index);
        }
        if group.modules.is_empty() || !Self::can_fit(&group.sizes, &cache_groups[cache as usize]) {
          self.discard(index);
        }
        continue;
      }
      // Each prepared candidate has only one residual state. Its native copy
      // already handles the unchanged branch; cloning both branches here makes
      // successive removals multiply the candidate set.
      self.keys.remove(&group.key);
      if removed.len() < group.chunks.len() {
        for chunk in removed {
          group.chunks.remove(chunk);
        }
      } else {
        group.chunks.retain(|chunk| !removed.contains(chunk));
      }
      group.key = ModuleGroupKey::Anonymous {
        cache_group_index: cache,
        chunks_key: get_key(group.chunks.iter().copied(), indices),
      };
      self.shrunk += 1;
      if self.keys.contains_key(&group.key) {
        let candidate = self.groups[index].take().expect("indexed candidate");
        for module in &candidate.modules {
          self
            .members
            .get_mut(module)
            .expect("member index")
            .remove(&index);
        }
        self.insert(candidate, sizes);
      } else {
        self.keys.insert(group.key.clone(), index);
      }
    }
  }

  pub(super) fn finish(self, plugin: &SplitChunksPlugin, sizes: &ModuleSizes) -> ModuleGroupMap {
    let mut result = ModuleGroupMap::default();
    let mut groups = self
      .groups
      .into_iter()
      .flatten()
      .filter(|group| group.changed && !group.modules.is_empty())
      .collect::<Vec<_>>();
    groups.sort_unstable_by(|a, b| a.key.cmp(&b.key));
    let dirty = groups.len();
    for candidate in groups {
      let (cache, name) = match &candidate.key {
        ModuleGroupKey::Named {
          cache_group_index,
          chunk_name,
        } => (*cache_group_index, Some(chunk_name.clone())),
        ModuleGroupKey::Anonymous {
          cache_group_index, ..
        } => (*cache_group_index, None),
      };
      let config = &plugin.cache_groups[cache as usize];
      let mut group = ModuleGroup::new(name, cache, config);
      if let Some(sources) = candidate.named_sources {
        let mut common = sources.values();
        let Some(first) = common.next() else { continue };
        let mut chunks = first.clone();
        for source in common {
          chunks.retain(|chunk| source.contains(chunk));
        }
        if chunks.is_empty() || chunks.len() < config.min_chunks as usize {
          continue;
        }
        for module in candidate.modules {
          group.add_module(module, chunks.iter().copied());
        }
      } else {
        group.modules = candidate.modules;
        group.chunks = candidate.chunks;
      }
      group.prepare_modules_for_sizes_and_compare();
      result.insert(candidate.key, group);
    }
    plugin.ensure_min_size_fit(&mut result, sizes);
    if std::env::var_os("RSPACK_RESIDUAL_AUDIT").is_some() {
      eprintln!(
        "[residual-delta] {{\"affected\":{},\"shrunk\":{},\"dirty\":{},\"eligible\":{}}}",
        self.affected,
        self.shrunk,
        dirty,
        result.len()
      );
    }
    result
  }
}

/// Each replacement copy must retain the union of the consumers of its removed
/// copies. Also check existing target modules when extending a reused target.
pub(super) fn placement_preserves_copies(
  placed: &ModuleChunkMap,
  source_chunks: &FxHashSet<ChunkUkey>,
  target: ChunkUkey,
  compilation: &Compilation,
) -> bool {
  let artifact = &compilation.build_chunk_graph_artifact;
  let target_chunk = artifact.chunk_by_ukey.expect_get(&target);
  let consumers = |chunks: &FxHashSet<ChunkUkey>| -> FxHashSet<ChunkGroupUkey> {
    chunks
      .iter()
      .flat_map(|chunk| {
        artifact
          .chunk_by_ukey
          .expect_get(chunk)
          .groups()
          .iter()
          .copied()
      })
      .collect()
  };
  let mut after = consumers(source_chunks);
  after.extend(target_chunk.groups().iter().copied());
  let mut modules = artifact
    .chunk_graph
    .get_chunk_modules_identifier(&target)
    .clone();
  match placed {
    ModuleChunkMap::Shared {
      modules: moving, ..
    } => modules.extend(moving.iter().copied()),
    ModuleChunkMap::ByModule(moving) => modules.extend(moving.keys().copied()),
  }
  let mut checked_shared = false;
  for module in &modules {
    let in_target = artifact.chunk_graph.is_module_in_chunk(module, target);
    if let Some(sources) = placed.get(module) {
      if matches!(placed, ModuleChunkMap::Shared { .. }) && !in_target {
        if checked_shared {
          continue;
        }
        checked_shared = true;
      }
      let mut before = consumers(sources);
      if in_target {
        before.extend(target_chunk.groups().iter().copied());
      }
      if before != after {
        return false;
      }
    } else {
      let before: FxHashSet<_> = target_chunk.groups().iter().copied().collect();
      if before != after {
        return false;
      }
    }
  }
  !modules.is_empty()
}
