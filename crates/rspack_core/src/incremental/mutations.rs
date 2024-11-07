use std::hash::{Hash, Hasher};

use once_cell::sync::OnceCell;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rspack_collections::{IdentifierDashMap, IdentifierMap, IdentifierSet, UkeySet};
use rspack_util::fx_hash::FxIndexSet;
use rustc_hash::FxHasher;

use crate::{
  AffectType, ChunkGraph, ChunkUkey, Compilation, Module, ModuleGraph, ModuleGraphConnection,
  ModuleIdentifier,
};

#[derive(Debug, Default)]
pub struct Mutations {
  inner: Vec<Mutation>,

  affected_modules_with_module_graph: OnceCell<IdentifierSet>,
  affected_modules_with_chunk_graph: OnceCell<IdentifierSet>,
  modules_with_chunk_graph_cache: IdentifierDashMap<Option<u64>>,
  affected_chunks_with_chunk_graph: OnceCell<UkeySet<ChunkUkey>>,
}

#[derive(Debug)]
pub enum Mutation {
  ModuleBuild { module: ModuleIdentifier },
  ModuleRevoke { module: ModuleIdentifier },
  ModuleSetAsync { module: ModuleIdentifier },
}

impl Mutations {
  pub fn add(&mut self, mutation: Mutation) {
    self.inner.push(mutation);
  }
}

impl Mutations {
  pub fn iter(&self) -> std::slice::Iter<Mutation> {
    self.inner.iter()
  }
}

pub struct IntoIter {
  inner: std::vec::IntoIter<Mutation>,
}

impl Iterator for IntoIter {
  type Item = Mutation;

  fn next(&mut self) -> Option<Self::Item> {
    self.inner.next()
  }
}

impl IntoIterator for Mutations {
  type Item = Mutation;
  type IntoIter = IntoIter;

  fn into_iter(self) -> Self::IntoIter {
    IntoIter {
      inner: self.inner.into_iter(),
    }
  }
}

impl Extend<Mutation> for Mutations {
  fn extend<T: IntoIterator<Item = Mutation>>(&mut self, iter: T) {
    self.inner.extend(iter);
  }
}

impl Mutations {
  pub fn get_affected_modules_with_module_graph(
    &self,
    module_graph: &ModuleGraph,
  ) -> IdentifierSet {
    self
      .affected_modules_with_module_graph
      .get_or_init(|| {
        compute_affected_modules_with_module_graph(
          module_graph,
          self
            .iter()
            .filter_map(|mutation| match mutation {
              Mutation::ModuleBuild { module } => Some(*module),
              _ => None,
            })
            .collect(),
        )
      })
      .clone()
  }

  pub fn get_affected_modules_with_chunk_graph(&self, compilation: &Compilation) -> IdentifierSet {
    self
      .affected_modules_with_chunk_graph
      .get_or_init(|| {
        compute_affected_modules_with_chunk_graph(
          self.get_affected_modules_with_module_graph(&compilation.get_module_graph()),
          self
            .iter()
            .filter_map(|mutation| match mutation {
              Mutation::ModuleRevoke { module } => Some(*module),
              _ => None,
            })
            .collect(),
          &self.modules_with_chunk_graph_cache,
          compilation,
        )
      })
      .clone()
  }

  pub fn get_affected_chunks_with_chunk_graph(
    &self,
    compilation: &Compilation,
  ) -> UkeySet<ChunkUkey> {
    dbg!(self
      .affected_chunks_with_chunk_graph
      .get_or_init(|| {
        compute_affected_chunks_with_chunk_graph(
          self.get_affected_modules_with_chunk_graph(compilation),
          &compilation.chunk_graph,
        )
      })
      .clone())
  }
}

#[tracing::instrument(skip_all)]
fn compute_affected_modules_with_module_graph(
  module_graph: &ModuleGraph,
  built_modules: IdentifierSet,
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

  enum AffectedModuleKind {
    Direct(ModuleIdentifier),
    Transitive(ModuleIdentifier),
  }

  let mut all_affected_modules: IdentifierSet = built_modules.clone();
  let affected_modules_cache_iter = built_modules.par_iter().flat_map(|module_identifier| {
    module_graph
      .get_incoming_connections_by_origin_module(module_identifier)
      .into_iter()
      .filter_map(|(referencing_module, connections)| {
        let referencing_module = referencing_module?;
        if all_affected_modules.contains(&referencing_module) {
          return None;
        }
        match reduce_affect_type(module_graph, &connections) {
          AffectType::False => None,
          AffectType::True => Some(AffectedModuleKind::Direct(referencing_module)),
          AffectType::Transitive => Some(AffectedModuleKind::Transitive(referencing_module)),
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
        match reduce_affect_type(module_graph, &connections) {
          AffectType::False => continue,
          AffectType::True => {
            direct_affected_modules.insert(referencing_module);
          }
          AffectType::Transitive => {
            transitive_affected_modules.insert(referencing_module);
          }
        };
      }
    }
  }
  all_affected_modules.extend(direct_affected_modules);
  all_affected_modules
}

#[tracing::instrument(skip_all)]
fn compute_affected_modules_with_chunk_graph(
  updated_modules: IdentifierSet,
  revoked_modules: IdentifierSet,
  cache: &IdentifierDashMap<Option<u64>>,
  compilation: &Compilation,
) -> IdentifierSet {
  #[tracing::instrument(skip_all, fields(module = ?module.identifier()))]
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

  for module in revoked_modules {
    cache.remove(&module);
  }
  for module in updated_modules {
    cache.insert(module, None);
  }

  let module_graph = compilation.get_module_graph();
  let affected_modules: IdentifierMap<u64> = cache
    .par_iter()
    .filter_map(|item| {
      let (module_identifier, &old_invalidate_key) = item.pair();
      let module = module_graph
        .module_by_identifier(module_identifier)
        .expect("should have module");
      let invalidate_key = create_chunk_graph_invalidate_key(
        &compilation.chunk_graph,
        &module_graph,
        compilation,
        module.as_ref(),
      );
      if old_invalidate_key.is_none()
        || matches!(old_invalidate_key, Some(old) if old != invalidate_key)
      {
        Some((*module_identifier, invalidate_key))
      } else {
        None
      }
    })
    .collect();
  for (&module_identifier, &invalidate_key) in affected_modules.iter() {
    cache.insert(module_identifier, Some(invalidate_key));
  }
  affected_modules.keys().copied().collect()
}

fn compute_affected_chunks_with_chunk_graph(
  updated_modules: IdentifierSet,
  chunk_graph: &ChunkGraph,
) -> UkeySet<ChunkUkey> {
  dbg!(&updated_modules);
  updated_modules
    .into_iter()
    .flat_map(|m| chunk_graph.get_module_chunks(m).iter().copied())
    .collect()
}
