use std::mem;

use futures::Future;
use indexmap::IndexMap;
use rspack_collections::{IdentifierIndexMap, IdentifierMap};
use rspack_error::Result;
use rspack_util::tracing_preset::TRACING_BENCH_TARGET;
use rustc_hash::FxHashMap as HashMap;
use tracing::instrument;

use crate::{
  ArtifactExt, ChunkByUkey, ChunkGraph, ChunkGroupByUkey, ChunkGroupUkey, ChunkUkey, Compilation,
  Logger, ModuleIdentifier,
  build_chunk_graph::code_splitter::{CodeSplitter, DependenciesBlockIdentifier},
  incremental::{IncrementalPasses, Mutation},
};

#[derive(Debug, Default)]
pub struct BuildChunkGraphArtifact {
  pub chunk_by_ukey: ChunkByUkey,
  pub chunk_graph: ChunkGraph,
  pub chunk_group_by_ukey: ChunkGroupByUkey,
  pub entrypoints: IndexMap<String, ChunkGroupUkey>,
  pub async_entrypoints: Vec<ChunkGroupUkey>,
  pub named_chunk_groups: HashMap<String, ChunkGroupUkey>,
  pub named_chunks: HashMap<String, ChunkUkey>,
  pub(crate) code_splitter: CodeSplitter,
  pub module_idx: IdentifierMap<(u32, u32)>,
}

impl BuildChunkGraphArtifact {
  // we can skip rebuilding chunk graph if none of modules
  // has changed its outgoings
  // we don't need to check if module has changed its incomings
  // if it changes, the incoming module changes its outgoings as well
  fn can_skip_rebuilding(&self, this_compilation: &Compilation) -> bool {
    self.can_skip_rebuilding_legacy(this_compilation)
  }

  fn can_skip_rebuilding_legacy(&self, this_compilation: &Compilation) -> bool {
    let logger = this_compilation.get_logger("rspack.Compilation.codeSplittingCache");

    if !this_compilation.entries.keys().eq(
      this_compilation
        .build_chunk_graph_artifact
        .entrypoints
        .keys(),
    ) {
      logger.log("entrypoints change detected, rebuilding chunk graph");
      return false;
    }

    let Some(mutations) = this_compilation
      .incremental
      .mutations_read(IncrementalPasses::BUILD_MODULE_GRAPH)
    else {
      logger.log("incremental for build module graph disabled, rebuilding chunk graph");
      // if disable incremental for build module graph phase, we can't skip rebuilding
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
    let module_graph_cache = &this_compilation.module_graph_cache_artifact;
    let affected_modules = mutations.get_affected_modules_with_module_graph(module_graph);
    let previous_modules_map = &this_compilation
      .build_chunk_graph_artifact
      .code_splitter
      .block_modules_runtime_map;

    if previous_modules_map.is_empty() {
      logger.log("no cache detected, rebuilding chunk graph");
      return false;
    }

    for module in affected_modules {
      let outgoings: Vec<ModuleIdentifier> = {
        let mut res = vec![];
        let mut active_modules = IdentifierIndexMap::<Vec<_>>::default();
        let module = module_graph
          .module_graph_module_by_identifier(&module)
          .expect("should have module");
        module
          .all_dependencies
          .iter()
          .filter(|dep_id| {
            module_graph
              .dependency_by_id(dep_id)
              .as_module_dependency()
              .is_none_or(|module_dep| !module_dep.weak())
          })
          .filter_map(|dep| module_graph.connection_by_dependency_id(dep))
          .for_each(|conn| {
            let m = *conn.module_identifier();
            active_modules.entry(m).or_default().push(conn);
          });

        'outer: for (m, connections) in active_modules {
          for conn in connections {
            if conn
              .active_state(
                module_graph,
                None,
                module_graph_cache,
                &this_compilation.exports_info_artifact,
              )
              .is_not_false()
            {
              res.push(m);
              continue 'outer;
            }
          }
        }

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

        for (outgoing, state, _) in outgoings.iter() {
          // we must insert module even if state is false
          // because we need to keep the import order
          previous_modules
            .entry(*outgoing)
            .and_modify(|v| {
              if state.is_not_false() {
                *v = *state;
              }
            })
            .or_insert(*state);
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
        logger.log(format!("module outgoings change detected: {module}"));
        return false;
      }
    }

    true
  }

  fn reset_for_rebuild(&mut self) {
    self.chunk_by_ukey = Default::default();
    self.chunk_graph = Default::default();
    self.chunk_group_by_ukey = Default::default();
    self.entrypoints.clear();
    self.async_entrypoints.clear();
    self.named_chunk_groups.clear();
    self.named_chunks.clear();
    self.code_splitter = Default::default();
    self.module_idx.clear();
  }
}

#[instrument(name = "Compilation:code_splitting",target=TRACING_BENCH_TARGET, skip_all)]
pub(crate) async fn use_code_splitting_cache<'a, T, F>(
  compilation: &'a mut Compilation,
  task: T,
) -> Result<()>
where
  T: Fn(&'a mut Compilation) -> F,
  F: Future<Output = Result<&'a mut Compilation>>,
{
  if !compilation.incremental.enabled() {
    task(compilation).await?;
    return Ok(());
  }

  let incremental_code_splitting = compilation
    .incremental
    .passes_enabled(IncrementalPasses::BUILD_CHUNK_GRAPH);
  let no_change = incremental_code_splitting
    && compilation
      .build_chunk_graph_artifact
      .can_skip_rebuilding(compilation);

  if no_change {
    let module_idx = &compilation.build_chunk_graph_artifact.module_idx;
    let module_graph = compilation
      .build_module_graph_artifact
      .get_module_graph_mut();
    for (m, (pre, post)) in module_idx.iter() {
      let mgm = module_graph.module_graph_module_by_identifier_mut(m);
      mgm.pre_order_index = Some(*pre);
      mgm.post_order_index = Some(*post);
    }

    return Ok(());
  }

  // Cache is not used, clear recovered artifact to avoid stale chunk graph data.
  compilation.build_chunk_graph_artifact.reset_for_rebuild();

  let compilation = task(compilation).await?;
  let mg = compilation.get_module_graph();
  let mut map = IdentifierMap::default();
  for (mid, mgm) in mg.module_graph_modules() {
    let (Some(pre), Some(post)) = (mgm.pre_order_index, mgm.post_order_index) else {
      continue;
    };

    map.insert(*mid, (pre, post));
  }
  compilation.build_chunk_graph_artifact.module_idx = map;
  Ok(())
}

impl ArtifactExt for BuildChunkGraphArtifact {
  const PASS: IncrementalPasses = IncrementalPasses::BUILD_CHUNK_GRAPH;
  fn should_recover(incremental: &crate::incremental::Incremental) -> bool {
    incremental.passes_enabled(IncrementalPasses::BUILD_CHUNK_GRAPH)
  }
  fn recover(_incremental: &crate::incremental::Incremental, new: &mut Self, old: &mut Self) {
    new.code_splitter = mem::take(&mut old.code_splitter);
    rayon::scope(|s| {
      s.spawn(|_| new.chunk_by_ukey.clone_from(&old.chunk_by_ukey));
      s.spawn(|_| new.chunk_graph.clone_from(&old.chunk_graph));
      s.spawn(|_| new.chunk_group_by_ukey.clone_from(&old.chunk_group_by_ukey));

      s.spawn(|_| new.async_entrypoints.clone_from(&old.async_entrypoints));
      s.spawn(|_| new.named_chunk_groups.clone_from(&old.named_chunk_groups));
      s.spawn(|_| new.named_chunks.clone_from(&old.named_chunks));
      s.spawn(|_| {
        new.entrypoints.clone_from(&old.entrypoints);
        new.module_idx.clone_from(&old.module_idx);
      });
    });
  }
}
