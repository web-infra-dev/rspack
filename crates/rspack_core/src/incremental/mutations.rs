use std::fmt;

use itertools::{Either, Itertools};
use once_cell::sync::OnceCell;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rspack_collections::{IdentifierSet, UkeySet};

use crate::{
  AffectType, ChunkUkey, Compilation, ModuleGraph, ModuleGraphConnection, ModuleIdentifier,
};

#[derive(Debug, Default)]
pub struct Mutations {
  inner: Vec<Mutation>,

  affected_modules_with_module_graph: OnceCell<IdentifierSet>,
  affected_chunks_with_chunk_graph: OnceCell<UkeySet<ChunkUkey>>,
}

impl fmt::Display for Mutations {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "[")?;
    for mutation in self.iter() {
      writeln!(f, "{mutation},")?;
    }
    writeln!(f, "]")
  }
}

#[derive(Debug)]
pub enum Mutation {
  ModuleAdd { module: ModuleIdentifier },
  ModuleUpdate { module: ModuleIdentifier },
  ModuleRemove { module: ModuleIdentifier },
  ModuleSetAsync { module: ModuleIdentifier },
  ModuleSetId { module: ModuleIdentifier },
  ModuleSetHashes { module: ModuleIdentifier },
  ChunkSetId { chunk: ChunkUkey },
  ChunkAdd { chunk: ChunkUkey },
  ChunkSplit { from: ChunkUkey, to: ChunkUkey },
  ChunksIntegrate { to: ChunkUkey },
  ChunkRemove { chunk: ChunkUkey },
}

impl fmt::Display for Mutation {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Mutation::ModuleAdd { module } => write!(f, "add module {module}"),
      Mutation::ModuleUpdate { module } => write!(f, "update module {module}"),
      Mutation::ModuleRemove { module } => write!(f, "remove module {module}"),
      Mutation::ModuleSetAsync { module } => write!(f, "set async module {module}"),
      Mutation::ModuleSetId { module } => write!(f, "set id module {module}"),
      Mutation::ModuleSetHashes { module } => write!(f, "set hashes module {module}"),
      Mutation::ChunkSetId { chunk } => write!(f, "set id chunk {}", chunk.as_u32()),
      Mutation::ChunkAdd { chunk } => write!(f, "add chunk {}", chunk.as_u32()),
      Mutation::ChunkSplit { from, to } => {
        write!(f, "split chunk {} to {}", from.as_u32(), to.as_u32())
      }
      Mutation::ChunksIntegrate { to } => write!(f, "integrate chunks to {}", to.as_u32()),
      Mutation::ChunkRemove { chunk } => write!(f, "remove chunk {}", chunk.as_u32()),
    }
  }
}

impl Mutations {
  pub fn add(&mut self, mutation: Mutation) {
    self.inner.push(mutation);
  }

  pub fn len(&self) -> usize {
    self.inner.len()
  }

  pub fn is_empty(&self) -> bool {
    self.inner.is_empty()
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
              Mutation::ModuleAdd { module } => Some(*module),
              Mutation::ModuleUpdate { module } => Some(*module),
              _ => None,
            })
            .collect(),
        )
      })
      .clone()
  }

  pub fn get_affected_chunks_with_chunk_graph(
    &self,
    compilation: &Compilation,
  ) -> UkeySet<ChunkUkey> {
    self
      .affected_chunks_with_chunk_graph
      .get_or_init(|| {
        self.iter().fold(UkeySet::default(), |mut acc, mutation| {
          match mutation {
            Mutation::ModuleSetHashes { module } => {
              acc.extend(compilation.chunk_graph.get_module_chunks(*module));
            }
            Mutation::ChunkAdd { chunk } => {
              acc.insert(*chunk);
            }
            Mutation::ChunkSplit { from, to } => {
              acc.insert(*from);
              acc.insert(*to);
            }
            Mutation::ChunksIntegrate { to } => {
              acc.insert(*to);
            }
            Mutation::ChunkRemove { chunk } => {
              acc.remove(chunk);
            }
            _ => {}
          };
          acc
        })
      })
      .clone()
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
  let (mut direct_affected_modules, mut transitive_affected_modules): (
    IdentifierSet,
    IdentifierSet,
  ) = built_modules
    .par_iter()
    .flat_map(|module_identifier| {
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
    })
    .collect::<Vec<_>>()
    .iter()
    .partition_map(|reduce_type| match reduce_type {
      AffectedModuleKind::Direct(m) => Either::Left(*m),
      AffectedModuleKind::Transitive(m) => Either::Right(*m),
    });

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
