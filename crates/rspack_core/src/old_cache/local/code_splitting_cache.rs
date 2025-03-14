use futures::Future;
use indexmap::IndexMap;
use rspack_collections::{IdentifierIndexMap, IdentifierSet};
use rspack_error::Result;
use rustc_hash::FxHashMap as HashMap;
use tracing::instrument;

use crate::{
  build_chunk_graph::{
    code_splitter::{CodeSplitter, DependenciesBlockIdentifier},
    new_code_splitter::CodeSplitter as NewCodeSplitter,
  },
  incremental::{IncrementalPasses, Mutation},
  ChunkByUkey, ChunkGraph, ChunkGroupByUkey, ChunkGroupUkey, ChunkUkey, Compilation, Logger,
  ModuleIdentifier,
};

#[derive(Debug, Default)]
pub struct CodeSplittingCache {
  chunk_by_ukey: ChunkByUkey,
  chunk_graph: ChunkGraph,
  chunk_group_by_ukey: ChunkGroupByUkey,
  entrypoints: IndexMap<String, ChunkGroupUkey>,
  async_entrypoints: Vec<ChunkGroupUkey>,
  named_chunk_groups: HashMap<String, ChunkGroupUkey>,
  named_chunks: HashMap<String, ChunkUkey>,
  pub(crate) code_splitter: CodeSplitter,
  pub(crate) new_code_splitter: NewCodeSplitter,
  pub(crate) module_idx: HashMap<ModuleIdentifier, (u32, u32)>,
}

impl CodeSplittingCache {
  // we can skip rebuilding chunk graph if none of modules
  // has changed its outgoings
  // we don't need to check if module has changed its incomings
  // if it changes, the incoming module changes its outgoings as well
  fn can_skip_rebuilding(&self, this_compilation: &Compilation) -> bool {
    let logger = this_compilation.get_logger("rspack.Compilation.codeSplittingCache");

    let Some(mutations) = this_compilation
      .incremental
      .mutations_read(IncrementalPasses::MAKE)
    else {
      logger.log("incremental for make disabled, rebuilding chunk graph");
      // if disable incremental for make phase, we can't skip rebuilding
      return false;
    };

    // if we have module removal, we can't skip rebuilding
    if mutations
      .iter()
      .any(|mutation| matches!(mutation, Mutation::ModuleRemove { .. }))
    {
      logger.log("module removal detected, rebuilding chunk graph");
      return false;
    }

    let module_graph = this_compilation.get_module_graph();
    let affected_modules = mutations.get_affected_modules_with_module_graph(&module_graph);
    let previous_modules_map = &self.code_splitter.block_modules_runtime_map;

    if previous_modules_map.is_empty() {
      logger.log("no cache detected, rebuilding chunk graph");
      return false;
    }

    for module in affected_modules {
      let outgoings: Vec<ModuleIdentifier> = {
        let mut res = vec![];
        let mut visited = IdentifierSet::default();
        let mut active_modules = IdentifierSet::default();
        module_graph
          .get_outgoing_connections_in_order(&module)
          .filter_map(|dep| module_graph.connection_by_dependency_id(dep))
          .map(|conn| {
            let m = *conn.module_identifier();
            if conn.active_state(&module_graph, None).is_not_false() {
              active_modules.insert(m);
            }
            m
          })
          .collect::<Vec<_>>()
          .into_iter()
          .for_each(|m| {
            if active_modules.contains(&m) && visited.insert(m) {
              res.push(m);
            }
          });

        res
      };

      // get outgoings from all runtimes in the previous compilation
      let mut previous_modules = IdentifierIndexMap::default();
      let all_runtimes = previous_modules_map.values();
      let mut miss_in_previous = true;

      all_runtimes.for_each(|modules| {
        let Some(outgoings) = modules.get(&DependenciesBlockIdentifier::Module(module)) else {
          return;
        };
        miss_in_previous = false;

        for (outgoing, state, _) in outgoings {
          // we must insert module even if state is false
          // because we need to keep the import order
          previous_modules
            .entry(*outgoing)
            .and_modify(|v| {
              if state.is_not_false() {
                *v = state;
              }
            })
            .or_insert(state);
        }
      });

      if miss_in_previous {
        logger.log("new module detected, rebuilding chunk graph");
        return false;
      }

      if previous_modules
        .iter()
        .filter(|(_, conn_state)| conn_state.is_not_false())
        .map(|(m, _)| *m)
        .collect::<Vec<_>>()
        != outgoings.clone()
      {
        // we find one module's outgoings has changed
        // we cannot skip rebuilding
        logger.log("module outgoings change detected");
        return false;
      }
    }

    if !this_compilation
      .entries
      .keys()
      .eq(this_compilation.code_splitting_cache.entrypoints.keys())
    {
      return false;
    }

    true
  }
}

#[instrument(skip_all)]
pub(crate) async fn use_code_splitting_cache<'a, T, F>(
  compilation: &'a mut Compilation,
  task: T,
) -> Result<()>
where
  T: Fn(&'a mut Compilation) -> F,
  F: Future<Output = Result<&'a mut Compilation>>,
{
  if !compilation
    .incremental
    .can_read_mutations(IncrementalPasses::MAKE)
  {
    task(compilation).await?;
    return Ok(());
  }

  let parallel_code_splitting = compilation.options.experiments.parallel_code_splitting;
  // TODO: parallel_code_splitting is not supported with incremental code splitting for now
  let incremental_code_splitting = !parallel_code_splitting
    && compilation
      .incremental
      .can_read_mutations(IncrementalPasses::BUILD_CHUNK_GRAPH);
  let no_change = compilation
    .code_splitting_cache
    .can_skip_rebuilding(compilation);

  if incremental_code_splitting || no_change {
    let cache = &mut compilation.code_splitting_cache;
    rayon::scope(|s| {
      s.spawn(|_| compilation.chunk_by_ukey = cache.chunk_by_ukey.clone());
      s.spawn(|_| compilation.chunk_graph = cache.chunk_graph.clone());
      s.spawn(|_| compilation.chunk_group_by_ukey = cache.chunk_group_by_ukey.clone());
      s.spawn(|_| compilation.entrypoints = cache.entrypoints.clone());
      s.spawn(|_| compilation.async_entrypoints = cache.async_entrypoints.clone());
      s.spawn(|_| compilation.named_chunk_groups = cache.named_chunk_groups.clone());
      s.spawn(|_| compilation.named_chunks = cache.named_chunks.clone());
    });

    let module_idx = cache.module_idx.clone();
    let mut module_graph = compilation.get_module_graph_mut();
    for (m, (pre, post)) in module_idx {
      let Some(mgm) = module_graph.module_graph_module_by_identifier_mut(&m) else {
        continue;
      };

      mgm.pre_order_index = Some(pre);
      mgm.post_order_index = Some(post);
    }

    if !incremental_code_splitting && no_change {
      return Ok(());
    }
  }

  let compilation = task(compilation).await?;
  let cache = &mut compilation.code_splitting_cache;
  rayon::scope(|s| {
    s.spawn(|_| cache.chunk_by_ukey = compilation.chunk_by_ukey.clone());
    s.spawn(|_| cache.chunk_graph = compilation.chunk_graph.clone());
    s.spawn(|_| cache.chunk_group_by_ukey = compilation.chunk_group_by_ukey.clone());
    s.spawn(|_| cache.entrypoints = compilation.entrypoints.clone());
    s.spawn(|_| cache.async_entrypoints = compilation.async_entrypoints.clone());
    s.spawn(|_| cache.named_chunk_groups = compilation.named_chunk_groups.clone());
    s.spawn(|_| cache.named_chunks = compilation.named_chunks.clone());
  });

  let mg = compilation.get_module_graph();
  let mut map = HashMap::default();
  for m in mg.modules().keys() {
    let Some(mgm) = mg.module_graph_module_by_identifier(m) else {
      continue;
    };

    let (Some(pre), Some(post)) = (mgm.pre_order_index, mgm.post_order_index) else {
      continue;
    };

    map.insert(*m, (pre, post));
  }
  let cache = &mut compilation.code_splitting_cache;
  cache.module_idx = map;
  Ok(())
}
