use std::fmt;

use either::Either;
use once_cell::sync::OnceCell;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rspack_collections::IdentifierSet;
use rustc_hash::FxHashSet;

use crate::{
  AffectType, ChunkUkey, Compilation, Dependency, DependencyId, ModuleGraph, ModuleIdentifier,
};

#[derive(Debug, Default)]
pub struct Mutations {
  inner: Vec<Mutation>,

  affected_modules_with_module_graph: OnceCell<IdentifierSet>,
  affected_modules_with_chunk_graph: OnceCell<IdentifierSet>,
  affected_chunks_with_chunk_graph: OnceCell<FxHashSet<ChunkUkey>>,
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
  DependencyUpdate { dependency: DependencyId },
  ModuleSetAsync { module: ModuleIdentifier },
  ModuleSetId { module: ModuleIdentifier },
  ModuleSetHashes { module: ModuleIdentifier },
  ChunkSetId { chunk: ChunkUkey },
  ChunkAdd { chunk: ChunkUkey },
  ChunkSplit { from: ChunkUkey, to: ChunkUkey },
  ChunksIntegrate { to: ChunkUkey },
  ChunkRemove { chunk: ChunkUkey },
  ChunkSetHashes { chunk: ChunkUkey },
}

impl fmt::Display for Mutation {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Mutation::ModuleAdd { module } => write!(f, "add module {module}"),
      Mutation::ModuleUpdate { module } => write!(f, "update module {module}"),
      Mutation::ModuleRemove { module } => write!(f, "remove module {module}"),
      Mutation::DependencyUpdate { dependency } => {
        write!(f, "update dependency {}", dependency.as_u32())
      }
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
      Mutation::ChunkSetHashes { chunk } => write!(f, "set hashes chunk {}", chunk.as_u32()),
    }
  }
}

impl Mutations {
  pub fn add(&mut self, mutation: Mutation) {
    self.inner.push(mutation);
  }

  #[allow(clippy::len_without_is_empty)]
  pub fn len(&self) -> usize {
    self.inner.len()
  }

  /// Drop mutations whose target no longer resolves in `module_graph`, and
  /// reset the affected-set caches derived from the log.
  ///
  /// The build-module-graph phase records a mutation for every module and
  /// dependency it touched. A later `Compilation::rebuild_module` call may
  /// revoke some of those targets again: rebuilding a module replaces its
  /// whole dependency set and may orphan away modules that lost their last
  /// importer. The underlying tables are not rollback-capable, so the
  /// recorded ids can outlive the objects they point to, while later passes
  /// walk the mutation log against the live graph. Fold the rebuild into
  /// the record by discarding entries whose target is gone.
  ///
  /// `ModuleRemove` entries are kept: they carry no live object and are
  /// consumed to purge caches of the removed module.
  ///
  /// If an affected-set cache was already computed before this call (e.g. by
  /// an earlier `finishModules` tap in the same compilation), it may still
  /// contain the revoked targets, so the caches are reset to force a
  /// recompute from the filtered log.
  pub fn retain_alive(&mut self, module_graph: &ModuleGraph) {
    self.inner.retain(|mutation| match mutation {
      Mutation::DependencyUpdate { dependency } => {
        crate::module_graph::internal::try_dependency_by_id(module_graph, dependency).is_some()
      }
      Mutation::ModuleAdd { module }
      | Mutation::ModuleUpdate { module }
      | Mutation::ModuleSetAsync { module }
      | Mutation::ModuleSetId { module }
      | Mutation::ModuleSetHashes { module } => module_graph.module_by_identifier(module).is_some(),
      _ => true,
    });
    self.affected_modules_with_module_graph = OnceCell::new();
    self.affected_modules_with_chunk_graph = OnceCell::new();
    self.affected_chunks_with_chunk_graph = OnceCell::new();
  }
}

impl Mutations {
  pub fn iter(&self) -> std::slice::Iter<'_, Mutation> {
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
        let mut built_modules = IdentifierSet::default();
        let mut built_dependencies = FxHashSet::default();
        for mutation in self.iter() {
          match mutation {
            Mutation::ModuleAdd { module } | Mutation::ModuleUpdate { module } => {
              built_modules.insert(*module);
            }
            // TODO: For now we only consider updated dependencies, do we need to consider added dependencies?
            // normally added dependencies' parent module should already in the built modules set (added modules
            // or updated modules), we can also add added dependencies here if there is issue or specific case
            // about that.
            Mutation::DependencyUpdate { dependency } => {
              built_dependencies.insert(*dependency);
            }
            _ => {}
          }
        }
        compute_affected_modules_with_module_graph(module_graph, built_modules, built_dependencies)
      })
      .clone()
  }

  pub fn get_affected_modules_with_chunk_graph(&self, compilation: &Compilation) -> IdentifierSet {
    self
      .affected_modules_with_chunk_graph
      .get_or_init(|| {
        let mg = compilation.get_module_graph();
        let mut modules = self.get_affected_modules_with_module_graph(mg);
        let mut chunks = FxHashSet::default();
        for mutation in self.iter() {
          match mutation {
            Mutation::ModuleSetAsync { module } => {
              modules.insert(*module);
            }
            Mutation::ModuleSetId { module } => {
              modules.insert(*module);
              modules.extend(
                mg.get_incoming_connections(module)
                  .filter_map(|c| c.original_module_identifier),
              );
            }
            Mutation::ChunkAdd { chunk } => {
              chunks.insert(chunk);
            }
            Mutation::ChunkRemove { chunk } => {
              chunks.remove(chunk);
            }
            Mutation::ChunkSetId { chunk } => {
              let chunk = compilation
                .build_chunk_graph_artifact
                .chunk_by_ukey
                .expect_get(chunk);
              modules.extend(
                chunk
                  .groups()
                  .iter()
                  .flat_map(|group| {
                    let group = compilation
                      .build_chunk_graph_artifact
                      .chunk_group_by_ukey
                      .expect_get(group);
                    group.origins()
                  })
                  .filter_map(|origin| origin.module),
              );
            }
            _ => {}
          }
        }
        modules.extend(chunks.into_iter().flat_map(|chunk| {
          compilation
            .build_chunk_graph_artifact
            .chunk_graph
            .get_chunk_modules_identifier(chunk)
        }));
        modules
      })
      .clone()
  }

  pub fn get_affected_chunks_with_chunk_graph(
    &self,
    compilation: &Compilation,
  ) -> FxHashSet<ChunkUkey> {
    self
      .affected_chunks_with_chunk_graph
      .get_or_init(|| {
        self.iter().fold(FxHashSet::default(), |mut acc, mutation| {
          match mutation {
            Mutation::ModuleSetHashes { module } => {
              acc.extend(
                compilation
                  .build_chunk_graph_artifact
                  .chunk_graph
                  .get_module_chunks(*module),
              );
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
            Mutation::ChunkSetId { chunk } => {
              acc.insert(*chunk);
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
  built_dependencies: FxHashSet<DependencyId>,
) -> IdentifierSet {
  fn reduce_affect_type<'a>(dependencies: impl Iterator<Item = &'a dyn Dependency>) -> AffectType {
    let mut affected = AffectType::False;
    for dependency in dependencies {
      match dependency.could_affect_referencing_module() {
        AffectType::True => affected = AffectType::True,
        AffectType::False => {}
        AffectType::Transitive => return AffectType::Transitive,
      }
    }
    affected
  }

  fn get_direct_and_transitive_affected_modules(
    modules: &IdentifierSet,
    all_affected_modules: &IdentifierSet,
    module_graph: &ModuleGraph,
  ) -> (IdentifierSet, IdentifierSet) {
    modules
      .par_iter()
      .flat_map(|module_identifier| {
        module_graph
          .get_incoming_connections_by_origin_module(module_identifier)
          .modules()
          .iter()
          .filter_map(|(referencing_module, connections)| {
            if all_affected_modules.contains(referencing_module) {
              return None;
            }
            match reduce_affect_type(
              connections
                .iter()
                .map(|c| module_graph.dependency_by_id(&c.dependency_id)),
            ) {
              AffectType::False => None,
              AffectType::True => Some(AffectedModuleKind::Direct(*referencing_module)),
              AffectType::Transitive => Some(AffectedModuleKind::Transitive(*referencing_module)),
            }
          })
          .collect::<Vec<_>>()
      })
      .partition_map(|kind| match kind {
        AffectedModuleKind::Direct(module) => Either::Left(module),
        AffectedModuleKind::Transitive(module) => Either::Right(module),
      })
  }

  enum AffectedModuleKind {
    Direct(ModuleIdentifier),
    Transitive(ModuleIdentifier),
  }

  let mut all_affected_modules: IdentifierSet = built_modules.clone();
  let mut transitive_affected_modules: IdentifierSet = {
    let (direct_affected_modules, transitive_affected_modules) =
      get_direct_and_transitive_affected_modules(
        &built_modules,
        &all_affected_modules,
        module_graph,
      );
    all_affected_modules.extend(direct_affected_modules);
    transitive_affected_modules
  };

  // It's possible that a module is deleted and the incoming dependencies is revoked, and added to
  // process_dependencies tasks queue to update the incoming dependencies, but the incoming dependencies'
  // parent module is not added to build_module tasks queue so the parent module is not marked as updated
  // and the incremental won't know that, so here we need to reduce the updated dependencies and collect
  // their parent module into affected modules (The module is affected if there dependencies is updated).
  for dep_id in built_dependencies {
    let dep = module_graph.dependency_by_id(&dep_id);
    match dep.could_affect_referencing_module() {
      AffectType::False => {}
      AffectType::True => {
        let Some(module) = module_graph.get_parent_module(&dep_id) else {
          continue;
        };
        all_affected_modules.insert(*module);
      }
      AffectType::Transitive => {
        let Some(module) = module_graph.get_parent_module(&dep_id) else {
          continue;
        };
        transitive_affected_modules.insert(*module);
      }
    };
  }

  while !transitive_affected_modules.is_empty() {
    let transitive_affected_modules_current = std::mem::take(&mut transitive_affected_modules);
    all_affected_modules.extend(transitive_affected_modules_current.iter().copied());
    let (direct_affected_modules, new_transitive_affected_modules) =
      get_direct_and_transitive_affected_modules(
        &transitive_affected_modules_current,
        &all_affected_modules,
        module_graph,
      );
    all_affected_modules.extend(direct_affected_modules);
    transitive_affected_modules.extend(new_transitive_affected_modules);
  }
  all_affected_modules
}

#[cfg(test)]
mod tests {
  use super::{Mutation, Mutations};
  use crate::{
    Context, Dependency, ModuleExt, ModuleGraph, ModuleIdentifier, RawModule, RuntimeGlobals,
    dependency::{LoaderImportDependency, UniqueDependency},
  };

  fn add_module(module_graph: &mut ModuleGraph, identifier: &str) {
    let module_identifier = ModuleIdentifier::from(identifier);
    module_graph.add_module(
      RawModule::new(
        String::new(),
        module_identifier,
        identifier.to_owned(),
        RuntimeGlobals::default(),
      )
      .boxed(),
    );
    module_graph.add_module_graph_module(crate::ModuleGraphModule::new(module_identifier));
  }

  #[test]
  fn retain_alive_drops_mutations_for_removed_targets_and_keeps_module_remove() {
    // Allocating dependency ids requires a compiler context on the thread.
    rspack_tasks::within_compiler_context_for_testing_sync(|| {
      retain_alive_drops_mutations_for_removed_targets_and_keeps_module_remove_inner()
    })
  }

  fn retain_alive_drops_mutations_for_removed_targets_and_keeps_module_remove_inner() {
    let mut module_graph = ModuleGraph::default();
    add_module(&mut module_graph, "alive");

    let alive_dependency = LoaderImportDependency::new("alive".to_owned(), Context::from(""));
    let alive_dependency_id = *alive_dependency.id();
    module_graph.add_dependency(UniqueDependency::new(alive_dependency));

    // Simulate a dependency revoked by a later `rebuild_module`: it is gone
    // from the dependency table while a recorded mutation may still point to it.
    let revoked_dependency = LoaderImportDependency::new("revoked".to_owned(), Context::from(""));
    let revoked_dependency_id = *revoked_dependency.id();
    module_graph.add_dependency(UniqueDependency::new(revoked_dependency));
    module_graph.revoke_dependency(&revoked_dependency_id, true);

    let mut mutations = Mutations::default();
    mutations.add(Mutation::DependencyUpdate {
      dependency: alive_dependency_id,
    });
    mutations.add(Mutation::DependencyUpdate {
      dependency: revoked_dependency_id,
    });
    mutations.add(Mutation::ModuleUpdate {
      module: ModuleIdentifier::from("alive"),
    });
    mutations.add(Mutation::ModuleUpdate {
      module: ModuleIdentifier::from("revoked"),
    });
    mutations.add(Mutation::ModuleAdd {
      module: ModuleIdentifier::from("revoked"),
    });
    mutations.add(Mutation::ModuleSetAsync {
      module: ModuleIdentifier::from("revoked"),
    });
    mutations.add(Mutation::ModuleSetId {
      module: ModuleIdentifier::from("revoked"),
    });
    mutations.add(Mutation::ModuleSetHashes {
      module: ModuleIdentifier::from("revoked"),
    });
    mutations.add(Mutation::ModuleRemove {
      module: ModuleIdentifier::from("revoked"),
    });

    // Simulate a consumer that derived the affected set before the rebuild:
    // the caches must be invalidated so they recompute from the filtered log.
    mutations
      .affected_modules_with_module_graph
      .set([ModuleIdentifier::from("revoked")].into_iter().collect())
      .ok()
      .expect("cache should be unset before retain_alive");

    mutations.retain_alive(&module_graph);

    // `ModuleRemove` carries no live object and must survive; everything
    // pointing at a revoked dependency or module must be dropped.
    assert_eq!(mutations.len(), 3);
    assert!(mutations.iter().any(
      |mutation| matches!(mutation, Mutation::DependencyUpdate { dependency } if *dependency == alive_dependency_id),
    ));
    assert!(!mutations.iter().any(
      |mutation| matches!(mutation, Mutation::DependencyUpdate { dependency } if *dependency == revoked_dependency_id),
    ));
    assert!(!mutations.iter().any(
      |mutation| matches!(mutation, Mutation::ModuleAdd { module } | Mutation::ModuleUpdate { module }
        | Mutation::ModuleSetAsync { module } | Mutation::ModuleSetId { module } | Mutation::ModuleSetHashes { module }
        if *module == ModuleIdentifier::from("revoked")),
    ));
    assert!(mutations.iter().any(
      |mutation| matches!(mutation, Mutation::ModuleRemove { module } if *module == ModuleIdentifier::from("revoked")),
    ));
    // The pre-computed affected set contained the revoked module; after the
    // rebuild reconciliation it must recompute without it.
    let affected = mutations.get_affected_modules_with_module_graph(&module_graph);
    assert!(!affected.contains(&ModuleIdentifier::from("revoked")));
    assert!(affected.contains(&ModuleIdentifier::from("alive")));
  }
}
