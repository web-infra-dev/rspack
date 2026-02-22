use std::{hash::BuildHasherDefault, sync::Arc};

use futures::future::BoxFuture;
use rayon::{iter::Either, prelude::*};
use rspack_collections::{IdentifierIndexSet, IdentifierSet};
use rspack_core::{
  BoxModule, Compilation, Filename, Module, ModuleGraph, ModuleIdentifier, SourceType,
};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_plugin_split_chunks::{
  CacheGroup, CacheGroupTest, CacheGroupTestFnCtx, ChunkNameGetter, ChunkNameGetterFnCtx,
  ModuleSizes, SplitChunkSizes, get_module_sizes, min_size::ModulesContainer,
};
use rspack_util::fx_hash::FxHashMap as HashMap;

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

#[derive(Default)]
struct MatchGroup {
  modules: IdentifierSet,
  removed: Vec<ModuleIdentifier>,
  added: Vec<ModuleIdentifier>,
  sizes: SplitChunkSizes,
  source_types_modules: HashMap<SourceType, IdentifierSet>,
  total_size: f64,
  name_hint: Option<String>,
  filename_template: Option<Filename>,
}

impl MatchGroup {
  pub fn add_module(&mut self, module: ModuleIdentifier) -> bool {
    let inserted = self.modules.insert(module);

    if inserted {
      self.added.push(module);
    }

    inserted
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
    .module_by_identifier(&module)
    .expect("should have module")
    .get_dependencies()
    .iter()
    .filter_map(|dep_id| module_graph.module_identifier_by_dependency_id(dep_id))
    .copied()
    .collect()
}

async fn matches_module_to_cache_group(
  module: &dyn Module,
  compilation: &Compilation,
  cache_group: &CacheGroup,
) -> Result<bool> {
  // match test
  let satisfied_test = match &cache_group.test {
    CacheGroupTest::String(str) => module.identifier().contains(str),
    CacheGroupTest::RegExp(regexp) => regexp.test(module.identifier().as_str()),
    CacheGroupTest::Fn(f) => f(CacheGroupTestFnCtx {
      compilation,
      module,
    })
    .await
    .to_rspack_result()?
    .unwrap_or(false),
    CacheGroupTest::Enabled => true,
  };

  if !satisfied_test {
    return Ok(false);
  }

  // match r#type
  if !(cache_group.r#type)(module) {
    return Ok(false);
  }

  // match layer
  if !(cache_group.layer)(module.get_layer().map(ToString::to_string))
    .await
    .to_rspack_result()
    .unwrap_or(false)
  {
    return Ok(false);
  }

  Ok(true)
}

pub(crate) async fn split(groups: &[CacheGroup], compilation: &mut Compilation) -> Result<()> {
  let modules = compilation.build_chunk_graph_artifact.chunk_graph.modules();
  let results: Vec<std::result::Result<_, _>> = rspack_futures::scope::<_, Result<_>>(|token| {
    modules.iter().copied().for_each(|module_identifier| {
      // SAFETY: `groups` and `compilation` outlive the scope and are only read (not mutated) concurrently.
      let s = unsafe { token.used((groups, &*compilation)) };
      s.spawn(move |(groups, compilation)| async move {
        let module_graph = compilation.get_module_graph();
        let module: &dyn Module = module_graph
          .module_by_identifier(&module_identifier)
          .expect("should have module")
          .as_ref();
        for (index, group) in groups.iter().enumerate() {
          if !matches_module_to_cache_group(module, compilation, group).await? {
            continue;
          }

          return Ok(match &group.name {
            ChunkNameGetter::String(name) => Some((Either::Left(name.clone()), module_identifier)),
            ChunkNameGetter::Disabled => Some((Either::Right(index), module_identifier)),
            ChunkNameGetter::Fn(func) => {
              let name_res = func(ChunkNameGetterFnCtx {
                module,
                compilation,
                chunks: &compilation
                  .build_chunk_graph_artifact
                  .chunk_graph
                  .get_module_chunks(module_identifier)
                  .iter()
                  .copied()
                  .collect(),
                cache_group_key: &group.key,
              })
              .await;

              match name_res {
                Ok(Some(name)) => Some((Either::Left(name), module_identifier)),
                Ok(None) => Some((Either::Right(index), module_identifier)),
                Err(err) => return Err(err),
              }
            }
          });
        }
        Ok(None)
      });
    });
  })
  .await;

  let modules = compilation.build_chunk_graph_artifact.chunk_graph.modules();
  let mut modules_in_group: IdentifierIndexSet =
    IdentifierIndexSet::with_capacity_and_hasher(modules.len(), BuildHasherDefault::default());
  let mut group_modules: HashMap<Either<String, usize>, MatchGroup> =
    HashMap::with_capacity_and_hasher(results.len(), Default::default());

  for item in results {
    let Some((index_or_name, module_identifier)) = item.to_rspack_result()?? else {
      continue;
    };
    modules_in_group.insert(module_identifier);

    let cache_group = match &index_or_name {
      Either::Left(_) => None,
      Either::Right(index) => Some(&groups[*index]),
    };

    let group = group_modules.entry(index_or_name).or_default();

    group.add_module(module_identifier);
    if let Some(cache_group) = &cache_group {
      group.name_hint = Some(cache_group.key.clone());
      group.filename_template = cache_group.filename.clone();
    }
  }

  // module is guaranteed to be exist in only one group
  // we should merge modules' dependencies into the same group
  let module_graph = compilation.get_module_graph();
  let mut group_order = group_modules.keys().cloned().collect::<Vec<_>>();
  group_order.sort_by(|a, b| match (a, b) {
    (Either::Left(la), Either::Left(lb)) => la.cmp(lb),
    (Either::Right(ra), Either::Right(rb)) => ra.cmp(rb),
    (Either::Left(_), Either::Right(_)) => std::cmp::Ordering::Less,
    (Either::Right(_), Either::Left(_)) => std::cmp::Ordering::Greater,
  });

  for key in group_order {
    let Some(match_group) = group_modules.get_mut(&key) else {
      continue;
    };

    let mut stack: Vec<_> = match_group.modules.iter().copied().collect();
    while let Some(module_identifier) = stack.pop() {
      for dep in get_module_deps(module_identifier, module_graph) {
        if !modules_in_group.insert(dep) {
          continue;
        }
        if !match_group.add_module(dep) {
          continue;
        }
        stack.push(dep);
      }
    }
  }

  let module_sizes = get_module_sizes(modules.par_iter().copied(), compilation);

  // ensure min size fit
  let group_modules = group_modules
    .into_iter()
    .filter_map(|(index_or_name, mut match_group)| {
      if let Either::Right(index) = &index_or_name {
        let min_size = &groups[*index].min_size;
        if match_group.get_sizes(&module_sizes).smaller_than(min_size) {
          return None;
        }
      }

      Some((index_or_name, match_group))
    })
    .collect::<Vec<_>>();

  let mut splitted_modules = IdentifierSet::default();
  let entry_modules = compilation.entry_modules();
  for (index_or_name, mut match_group) in group_modules {
    match_group
      .modules
      .retain(|m| !splitted_modules.contains(m));

    if match_group.modules.is_empty() {
      continue;
    }

    // split chunk
    let chunk_name = match &index_or_name {
      Either::Left(name) => Some(name),
      Either::Right(_) => None,
    };

    let chunk_ukey = if let Some(chunk_name) = chunk_name {
      let (ukey, created) = Compilation::add_named_chunk(
        chunk_name.clone(),
        &mut compilation.build_chunk_graph_artifact.chunk_by_ukey,
        &mut compilation.build_chunk_graph_artifact.named_chunks,
      );

      if !created {
        compilation.push_diagnostic(rspack_error::Diagnostic::warn(
          String::new(),
          format!("Merge modules into a existing chunk: {chunk_name}. This can cause runtime errors if there are cyclic dependencies"),
        ));
        continue;
      }

      ukey
    } else {
      Compilation::add_chunk(&mut compilation.build_chunk_graph_artifact.chunk_by_ukey)
    };

    compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .add_chunk(chunk_ukey);

    let chunk = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .expect_get_mut(&chunk_ukey);
    if let Some(filename_template) = match_group.filename_template {
      chunk.set_filename_template(Some(filename_template));
    }

    if let Some(name_hint) = match_group.name_hint {
      chunk.add_id_name_hints(name_hint);
    }

    for m in &match_group.modules {
      let chunks = compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .get_module_chunks(*m);
      if chunks.is_empty() {
        continue;
      }

      let orig_chunk = EsmLibraryPlugin::get_module_chunk(*m, compilation);

      compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .connect_chunk_and_module(chunk_ukey, *m);

      compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .disconnect_chunk_and_module(&orig_chunk, *m);

      if entry_modules.contains(m) {
        compilation
          .build_chunk_graph_artifact
          .chunk_graph
          .disconnect_chunk_and_entry_module(&orig_chunk, *m);

        let entrypoints = compilation
          .build_chunk_graph_artifact
          .chunk_by_ukey
          .expect_get(&orig_chunk)
          .groups();
        for entrypoint in entrypoints {
          compilation
            .build_chunk_graph_artifact
            .chunk_graph
            .connect_chunk_and_entry_module(chunk_ukey, *m, *entrypoint);
        }
      }

      let [Some(chunk), Some(orig_chunk)] = compilation
        .build_chunk_graph_artifact
        .chunk_by_ukey
        .get_many_mut([&chunk_ukey, &orig_chunk])
      else {
        unreachable!()
      };

      orig_chunk.split(
        chunk,
        &mut compilation.build_chunk_graph_artifact.chunk_group_by_ukey,
      );
    }

    splitted_modules.extend(match_group.modules.clone());
  }

  Ok(())
}
