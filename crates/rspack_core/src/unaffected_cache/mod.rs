mod mutations;

use std::{
  hash::{BuildHasherDefault, Hash, Hasher},
  sync::Mutex,
};

pub use mutations::{Mutation, Mutations};
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use rspack_collections::{IdentifierDashMap, IdentifierHasher, IdentifierMap, IdentifierSet};
use rspack_util::fx_hash::FxIndexSet;
use rustc_hash::FxHasher;

use crate::{
  AffectType, ChunkGraph, Compilation, Module, ModuleGraph, ModuleGraphConnection, ModuleIdentifier,
};

#[derive(Debug, Default)]
pub struct UnaffectedModulesCache {
  module_to_cache: IdentifierDashMap<UnaffectedModuleCache>,
  affected_modules_with_module_graph: Mutex<IdentifierSet>,
  affected_modules_with_chunk_graph: Mutex<IdentifierSet>,
}

#[derive(Debug)]
struct UnaffectedModuleCache {
  module_graph_invalidate_key: u64,
  with_chunk_graph_cache: Option<UnaffectedModuleWithChunkGraphCache>,
}

#[derive(Debug)]
struct UnaffectedModuleWithChunkGraphCache {
  chunk_graph_invalidate_key: u64,
}

impl UnaffectedModulesCache {
  fn par_iter(
    &self,
  ) -> dashmap::rayon::map::Iter<
    ModuleIdentifier,
    UnaffectedModuleCache,
    BuildHasherDefault<IdentifierHasher>,
  > {
    self.module_to_cache.par_iter()
  }

  // #[tracing::instrument(skip_all, fields(module = ?key))]
  fn remove_cache(&self, key: &ModuleIdentifier) {
    self.module_to_cache.remove(key);
  }

  // #[tracing::instrument(skip_all, fields(module = ?key))]
  fn insert_cache(&self, key: ModuleIdentifier, value: UnaffectedModuleCache) {
    self.module_to_cache.insert(key, value);
  }

  // #[tracing::instrument(skip_all, fields(module = ?key))]
  fn affect_cache(&self, key: &ModuleIdentifier) -> Option<()> {
    let mut cache = self.module_to_cache.get_mut(key)?;
    cache.with_chunk_graph_cache = None;
    // remove other cache...
    Some(())
  }

  // #[tracing::instrument(skip_all, fields(module = ?key))]
  fn insert_chunk_graph_cache(
    &self,
    key: &ModuleIdentifier,
    value: UnaffectedModuleWithChunkGraphCache,
  ) -> Option<()> {
    let mut cache = self.module_to_cache.get_mut(key)?;
    cache.with_chunk_graph_cache = Some(value);
    Some(())
  }

  // #[tracing::instrument(skip_all)]
  pub fn compute_affected_modules_with_module_graph(&self, compilation: &Compilation) {
    let mg = compilation.get_module_graph();
    let modules = mg.modules().keys().copied().collect();
    let mut affected_modules = self
      .affected_modules_with_module_graph
      .lock()
      .expect("failed to lock");
    *affected_modules = compute_affected_modules_with_module_graph(compilation, modules);
  }

  // #[tracing::instrument(skip_all)]
  pub fn get_affected_modules_with_module_graph(&self) -> &Mutex<IdentifierSet> {
    &self.affected_modules_with_module_graph
  }

  // #[tracing::instrument(skip_all)]
  pub fn compute_affected_modules_with_chunk_graph(&self, compilation: &Compilation) {
    let mut affected_modules = self
      .affected_modules_with_chunk_graph
      .lock()
      .expect("failed to lock");
    *affected_modules = compute_affected_modules_with_chunk_graph(compilation);
  }

  // #[tracing::instrument(skip_all)]
  pub fn get_affected_modules_with_chunk_graph(&self) -> &Mutex<IdentifierSet> {
    &self.affected_modules_with_chunk_graph
  }
}

impl UnaffectedModuleCache {
  fn new(module_graph_invalidate_key: u64) -> Self {
    Self {
      module_graph_invalidate_key,
      with_chunk_graph_cache: None,
    }
  }

  // #[tracing::instrument(skip_all, fields(module = ?module.identifier()))]
  fn create_module_graph_invalidate_key(module_graph: &ModuleGraph, module: &dyn Module) -> u64 {
    let mut hasher = FxHasher::default();
    module
      .build_info()
      .expect("should have build_info after build")
      .hash
      .as_ref()
      .hash(&mut hasher);
    for dep_id in module_graph
      .get_ordered_connections(&module.identifier())
      .expect("should have module")
    {
      dep_id.hash(&mut hasher);
    }
    hasher.finish()
  }
}

impl UnaffectedModuleWithChunkGraphCache {
  // #[tracing::instrument(skip_all, fields(module = ?module.identifier()))]
  fn create_chunk_graph_invalidate_key(
    chunk_graph: &ChunkGraph,
    module_graph: &ModuleGraph,
    compilation: &Compilation,
    module: &dyn Module,
  ) -> u64 {
    let module_identifier = module.identifier();
    let mut hasher = FxHasher::default();
    chunk_graph
      .get_module_id(module_identifier)
      .hash(&mut hasher);
    let module_ids: FxIndexSet<_> = module_graph
      .get_ordered_connections(&module_identifier)
      .expect("should have module")
      .into_iter()
      .filter_map(|dep_id| {
        let connection = module_graph
          .connection_by_dependency_id(dep_id)
          .expect("should have connection");
        chunk_graph.get_module_id(*connection.module_identifier())
      })
      .collect();
    for module_id in module_ids {
      module_id.hash(&mut hasher);
    }
    for block_id in module.get_blocks() {
      let Some(chunk_group) =
        chunk_graph.get_block_chunk_group(block_id, &compilation.chunk_group_by_ukey)
      else {
        continue;
      };
      for chunk in &chunk_group.chunks {
        let chunk = compilation.chunk_by_ukey.expect_get(chunk);
        chunk.id.as_ref().hash(&mut hasher);
      }
    }
    hasher.finish()
  }
}

fn compute_affected_modules_with_module_graph(
  compilation: &Compilation,
  modules: IdentifierSet,
) -> IdentifierSet {
  fn reduce_affect_type(
    module_graph: &ModuleGraph,
    connections: &[ModuleGraphConnection],
  ) -> AffectType {
    let mut affected = AffectType::False;
    for connection in connections {
      let Some(dependency) = module_graph.dependency_by_id(&connection.dependency_id) else {
        continue;
      };
      match dependency.could_affect_referencing_module() {
        AffectType::True => affected = AffectType::True,
        AffectType::False => continue,
        AffectType::Transitive => return AffectType::Transitive,
      }
    }
    affected
  }

  enum ModulesCacheOp {
    Delete(ModuleIdentifier),
    Unaffected(ModuleIdentifier),
    Affected(ModuleIdentifier, u64),
  }

  let module_graph = compilation.get_module_graph();
  let modules_cache = compilation.unaffected_modules_cache.clone();
  let mut modules_without_cache = modules;
  let results: Vec<ModulesCacheOp> = modules_cache
    .par_iter()
    .map(|item| {
      let (module_identifier, cache) = item.pair();
      if modules_without_cache.contains(module_identifier) {
        let module = module_graph
          .module_by_identifier(module_identifier)
          .expect("should have module");
        let invalidate_key =
          UnaffectedModuleCache::create_module_graph_invalidate_key(&module_graph, module.as_ref());
        if cache.module_graph_invalidate_key != invalidate_key {
          ModulesCacheOp::Affected(*module_identifier, invalidate_key)
        } else {
          ModulesCacheOp::Unaffected(*module_identifier)
        }
      } else {
        ModulesCacheOp::Delete(*module_identifier)
      }
    })
    .collect();
  let mut affected_modules_cache = IdentifierMap::default();
  for result in results {
    match result {
      ModulesCacheOp::Delete(m) => {
        modules_cache.remove_cache(&m);
      }
      ModulesCacheOp::Unaffected(m) => {
        modules_without_cache.remove(&m);
      }
      ModulesCacheOp::Affected(m, invalidate_key) => {
        modules_without_cache.remove(&m);
        affected_modules_cache.insert(m, invalidate_key);
      }
    }
  }
  let more_affected_modules: Vec<_> = modules_without_cache
    .into_par_iter()
    .map(|module_identifier| {
      let module = module_graph
        .module_by_identifier(&module_identifier)
        .expect("should have module");
      let invalidate_key =
        UnaffectedModuleCache::create_module_graph_invalidate_key(&module_graph, module.as_ref());
      (module_identifier, invalidate_key)
    })
    .collect();
  affected_modules_cache.extend(more_affected_modules);

  enum AffectedModuleKind {
    Direct(ModuleIdentifier),
    Transitive(ModuleIdentifier),
  }
  let mut all_affected_modules: IdentifierSet = affected_modules_cache.keys().copied().collect();
  let affected_modules_cache_iter =
    affected_modules_cache
      .par_iter()
      .flat_map(|(&module_identifier, &invalidate_key)| {
        modules_cache.insert_cache(
          module_identifier,
          UnaffectedModuleCache::new(invalidate_key),
        );
        module_graph
          .get_incoming_connections_by_origin_module(&module_identifier)
          .into_iter()
          .filter_map(|(referencing_module, connections)| {
            let referencing_module = referencing_module?;
            if all_affected_modules.contains(&referencing_module) {
              return None;
            }
            match reduce_affect_type(&module_graph, &connections) {
              AffectType::False => None,
              AffectType::True => {
                modules_cache.affect_cache(&referencing_module);
                Some(AffectedModuleKind::Direct(referencing_module))
              }
              AffectType::Transitive => {
                modules_cache.affect_cache(&referencing_module);
                Some(AffectedModuleKind::Transitive(referencing_module))
              }
            }
          })
          .collect::<Vec<_>>()
      });
  let mut direct_affected_modules: IdentifierSet = affected_modules_cache_iter
    .clone()
    .filter_map(|k| match k {
      AffectedModuleKind::Direct(m) => Some(m),
      AffectedModuleKind::Transitive(_) => None,
    })
    .collect();
  let mut transitive_affected_modules: IdentifierSet = affected_modules_cache_iter
    .clone()
    .filter_map(|k| match k {
      AffectedModuleKind::Transitive(m) => Some(m),
      AffectedModuleKind::Direct(_) => None,
    })
    .collect();
  while !transitive_affected_modules.is_empty() {
    let transitive_affected_modules_current = std::mem::take(&mut transitive_affected_modules);
    all_affected_modules.extend(transitive_affected_modules_current.iter().copied());
    for &module_identifier in transitive_affected_modules_current.iter() {
      for (referencing_module, connections) in
        module_graph.get_incoming_connections_by_origin_module(&module_identifier)
      {
        let Some(referencing_module) = referencing_module else {
          continue;
        };
        if all_affected_modules.contains(&referencing_module) {
          continue;
        }
        match reduce_affect_type(&module_graph, &connections) {
          AffectType::False => continue,
          AffectType::True => {
            direct_affected_modules.insert(referencing_module);
          }
          AffectType::Transitive => {
            transitive_affected_modules.insert(referencing_module);
          }
        };
        modules_cache.affect_cache(&referencing_module);
      }
    }
  }
  all_affected_modules.extend(direct_affected_modules);
  all_affected_modules
}

fn compute_affected_modules_with_chunk_graph(compilation: &Compilation) -> IdentifierSet {
  let modules_cache = compilation.unaffected_modules_cache.clone();
  let module_graph = compilation.get_module_graph();
  let affected_modules: IdentifierMap<u64> = modules_cache
    .par_iter()
    .filter_map(|item| {
      let (module_identifier, cache) = item.pair();
      let module = module_graph
        .module_by_identifier(module_identifier)
        .expect("should have module");
      let invalidate_key = UnaffectedModuleWithChunkGraphCache::create_chunk_graph_invalidate_key(
        &compilation.chunk_graph,
        &module_graph,
        compilation,
        module.as_ref(),
      );
      if let Some(module_graph_cache) = &cache.with_chunk_graph_cache {
        if module_graph_cache.chunk_graph_invalidate_key != invalidate_key {
          Some((*module_identifier, invalidate_key))
        } else {
          None
        }
      } else {
        Some((*module_identifier, invalidate_key))
      }
    })
    .collect();
  for (module_identifier, invalidate_key) in affected_modules.iter() {
    modules_cache.insert_chunk_graph_cache(
      module_identifier,
      UnaffectedModuleWithChunkGraphCache {
        chunk_graph_invalidate_key: *invalidate_key,
      },
    );
  }
  affected_modules.keys().copied().collect()
}
