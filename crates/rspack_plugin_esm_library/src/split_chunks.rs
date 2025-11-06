use std::{hash::BuildHasherDefault, sync::Arc};

use futures::future::BoxFuture;
use rayon::{iter::Either, prelude::*};
use rspack_collections::IdentifierSet;
use rspack_core::{
  BoxModule, Compilation, CompilerId, Module, ModuleGraph, ModuleIdentifier, SourceType,
};
use rspack_error::Result;
use rspack_plugin_split_chunks::{
  ModuleSizes, SplitChunkSizes, get_module_sizes,
  min_size::{ModulesContainer, remove_min_size_violating_modules},
};
use rspack_util::fx_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::EsmLibraryPlugin;

pub type GetNameGetter = Either<
  Option<String>,
  Arc<
    dyn for<'a> Fn(&'a BoxModule, &'a Compilation) -> BoxFuture<'static, Result<Option<String>>>
      + Sync
      + Send,
  >,
>;
pub type ModuleFilter =
  Arc<dyn for<'a> Fn(&'a BoxModule, &'a Compilation) -> BoxFuture<'static, bool> + Sync + Send>;
pub type ModuleTypeFilter =
  Arc<dyn for<'a> Fn(&'a BoxModule, &'a Compilation) -> BoxFuture<'static, bool> + Sync + Send>;
use derive_more::Debug;

#[derive(Debug)]
pub struct CacheGroup {
  #[debug(skip)]
  pub name: GetNameGetter,
  pub key: String,
  #[debug(skip)]
  pub test: ModuleFilter,
  #[debug(skip)]
  pub r#type: ModuleTypeFilter,
  pub filename: Option<String>,
  pub priority: f64,
  pub min_size: Option<SplitChunkSizes>,
  pub index: usize,
}

impl CacheGroup {
  async fn matches(&self, module: &BoxModule, compilation: &Compilation) -> bool {
    (self.test)(module, compilation).await && (self.r#type)(module, compilation).await
  }
}

pub(crate) struct SplitChunks {
  cache_groups: Vec<CacheGroup>,
}

struct SplitResult {
  modules: IdentifierSet,
  name: Option<String>,
}

#[derive(Default)]
struct MatchGroup {
  modules: IdentifierSet,
  removed: Vec<ModuleIdentifier>,
  added: Vec<ModuleIdentifier>,
  sizes: SplitChunkSizes,

  source_types_modules: HashMap<SourceType, IdentifierSet>,
  total_size: f64,
}

impl MatchGroup {
  pub fn add_module(&mut self, module: ModuleIdentifier) {
    if self.modules.insert(module) {
      self.added.push(module);
    }
  }

  pub fn remove_module(&mut self, module: ModuleIdentifier) {
    if self.modules.remove(&module) {
      self.removed.push(module);
    }
  }

  pub fn get_sizes(&mut self, module_sizes: &ModuleSizes) -> SplitChunkSizes {
    if !self.added.is_empty() {
      let added = std::mem::take(&mut self.added);
      for module in added {
        let module_sizes = module_sizes.get(&module).expect("should have module size");
        for (ty, s) in module_sizes.iter() {
          let size = self.sizes.entry(*ty).or_default();
          *size += s;
          self.total_size += s;
          self
            .source_types_modules
            .entry(*ty)
            .or_default()
            .insert(module);
        }
      }
    }
    if !self.removed.is_empty() {
      let removed = std::mem::take(&mut self.removed);
      for module in removed {
        let module_sizes = module_sizes.get(&module).expect("should have module size");
        for (ty, s) in module_sizes.iter() {
          let size = self.sizes.entry(*ty).or_default();
          *size -= s;
          *size = size.max(0.0);
          self.total_size -= s;
          self
            .source_types_modules
            .entry(*ty)
            .or_default()
            .remove(&module);
        }
      }
    }

    self.sizes.clone()
  }
}

impl ModulesContainer for MatchGroup {
  fn get_sizes(&mut self, module_sizes: &ModuleSizes) -> SplitChunkSizes {
    MatchGroup::get_sizes(self, module_sizes)
  }

  fn get_source_types_modules(
    &self,
    ty: &[SourceType],
    module_sizes: &ModuleSizes,
  ) -> IdentifierSet {
    // if there is only one source type, we can just use the `source_types_modules` directly
    // instead of iterating over all modules
    if ty.len() == 1 {
      self
        .source_types_modules
        .get(ty.first().expect("should have at least one source type"))
        .cloned()
        .unwrap_or_default()
    } else {
      self
        .modules
        .iter()
        .filter_map(|module| {
          let sizes = module_sizes.get(module).expect("should have module size");
          if ty.iter().any(|ty| sizes.contains_key(ty)) {
            Some(*module)
          } else {
            None
          }
        })
        .collect()
    }
  }

  fn modules(&self) -> &IdentifierSet {
    &self.modules
  }

  fn remove_module(&mut self, module: ModuleIdentifier) {
    MatchGroup::remove_module(self, module);
  }
}

fn get_module_deps(module: ModuleIdentifier, module_graph: &ModuleGraph) -> Vec<ModuleIdentifier> {
  module_graph
    .get_outgoing_deps_in_order(&module)
    .filter_map(|dep_id| module_graph.module_identifier_by_dependency_id(dep_id))
    .copied()
    .collect()
}

pub(crate) async fn split(
  groups: &[CacheGroup],
  compilation: &mut Compilation,
  merge_dependencies: bool,
) -> Result<()> {
  let module_graph = compilation.get_module_graph();

  let modules = compilation.chunk_graph.modules();

  let results: Vec<Result<Option<Either>>> = rspack_futures::scope::<_, Result<_>>(|token| {
    modules.iter().for_each(|module_identifier| {
      let module = module_graph
        .module_by_identifier(module_identifier)
        .expect("should have module");
      let s = unsafe { token.used((groups, &*compilation)) };
      s.spawn(|(groups, compilation)| async move {
        for (index, group) in groups.iter().enumerate() {
          if !group.matches(&module, compilation).await {
            continue;
          }

          return match &group.name {
            Either::Left(Some(name)) => Some((Either::Left(name.clone()), *module_identifier)),
            Either::Left(None) => Some((Either::Right(index), *module_identifier)),
            Either::Right(func) => {
              if let Some(name) = func(&module, &compilation).await? {
                Some((Either::Left(name), *module_identifier))
              } else {
                Some((Either::Right(index), *module_identifier))
              }
            }
          };
        }
        None
      });
    });
  })
  .await;
  let results = results.into_iter().collect::<Result<Vec<_>>>();

  let mut modules_in_group: IdentifierSet =
    IdentifierSet::with_capacity_and_hasher(modules.len(), BuildHasherDefault::default());
  let mut group_modules: HashMap<Either<String, usize>, MatchGroup> =
    HashMap::with_capacity_and_hasher(results.len(), Default::default());

  for (index_or_name, module_identifier) in results {
    modules_in_group.insert(module_identifier);

    group_modules
      .entry(index_or_name)
      .or_default()
      .modules
      .insert(module_identifier);
  }

  // module is guarrented to be exist in only one group
  // we should merge modules' dependencies into the same group
  if merge_dependencies {
    group_modules.par_iter_mut().for_each(|(_, match_group)| {
      for m in match_group.modules.clone() {
        // merge dependencies

        let mut stack = get_module_deps(m, &module_graph);
        while let Some(m) = stack.pop() {
          // if module is already in any group, skip
          if modules_in_group.contains(&m) {
            continue;
          }

          // if module is already in the group, skip
          if !match_group.modules.insert(m) {
            continue;
          }

          stack.extend(get_module_deps(m, &module_graph));
        }
      }
    });
  }

  let module_sizes = get_module_sizes(modules.par_iter().copied(), compilation);

  // ensure min size fit
  let group_modules = group_modules
    .into_par_iter()
    .filter_map(|(index_or_name, mut match_group)| {
      if let Either::Right(index) = &index_or_name {
        if let Some(min_size) = &groups[*index].min_size {
          if remove_min_size_violating_modules(
            &groups[*index].key,
            &groups[*index].key,
            &mut match_group,
            min_size,
            &module_sizes,
          ) {
            return None;
          }

          // TODO:
          // we don't support maxSize yet, as the maxSize split algorithm is complex
          // it needs to split modules into multiple chunks if the size exceeds maxSize,
          // but this is likely causing cycles in chunks, and if so, there is a great possibility
          // causing runtime errors
          // for example, A -> B -> C
          // if the chunk split A and C into one chunk, and B into another chunk,
          // then there will be a cycle between these two chunks
        }
      }

      Some((index_or_name, match_group))
    })
    .collect::<Vec<_>>();

  for (index_or_name, match_group) in group_modules {
    if match_group.modules.is_empty() {
      continue;
    }

    let chunk_name = match &index_or_name {
      Either::Left(name) => Some(name),
      Either::Right(_) => None,
    };

    // split chunk
    let chunk_ukey = if let Some(chunk_name) = chunk_name {
      let (ukey, created) = Compilation::add_named_chunk(
        chunk_name.clone(),
        &mut compilation.chunk_by_ukey,
        &mut compilation.named_chunks,
      );

      if !created {
        compilation.push_diagnostic(rspack_error::Diagnostic::warn(
          "".into(),
          format!("Merge modules into a existing chunk: {}. This can cause runtime errors if there are cyclic dependencies", chunk_name),
        ));
      }

      ukey
    } else {
      Compilation::add_chunk(&mut compilation.chunk_by_ukey)
    };

    let entry_modules = compilation.entry_modules();

    for m in &match_group.modules {
      let chunks = compilation.chunk_graph.get_module_chunks(*m);
      if chunks.is_empty() {
        continue;
      }

      compilation
        .chunk_graph
        .connect_chunk_and_module(chunk_ukey, *m);

      let orig_chunk = EsmLibraryPlugin::get_module_chunk(*m, compilation);
      compilation
        .chunk_graph
        .disconnect_chunk_and_module(&orig_chunk, *m);

      if entry_modules.contains(m) {
        compilation
          .chunk_graph
          .disconnect_chunk_and_entry_module(&orig_chunk, *m);

        let entrypoints = compilation.chunk_by_ukey.expect_get(&orig_chunk).groups();
        for entrypoint in entrypoints {
          compilation
            .chunk_graph
            .connect_chunk_and_entry_module(chunk_ukey, *m, *entrypoint);
        }
      }

      let [Some(chunk), Some(orig_chunk)] = compilation
        .chunk_by_ukey
        .get_many_mut([&chunk_ukey, &orig_chunk])
      else {
        unreachable!()
      };

      orig_chunk.split(chunk, &mut compilation.chunk_group_by_ukey);
    }
  }

  Ok(())
}

fn ensure_min_size() {}
