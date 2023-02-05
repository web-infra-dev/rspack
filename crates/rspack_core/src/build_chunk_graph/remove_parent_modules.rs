// This is not a port of https://github.com/webpack/webpack/blob/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/optimize/RemoveParentModulesPlugin.js
// But they do the same thing.

use std::sync::Arc;

use petgraph::dot::Dot;
use rayon::prelude::ParallelBridge;
use rayon::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};

use super::code_splitter::CodeSplitter;
use crate::{ChunkUkey, Compilation, IdentifierSet, ModuleIdentifier};

type ChunkRelationGraph = petgraph::graphmap::DiGraphMap<ChunkUkey, ()>;
type DefinitelyLoadedModules = IdentifierSet;

#[derive(Debug, Default)]
pub(super) struct RemoveParentModulesContext {
  chunk_relation_graph: ChunkRelationGraph,
  root_chunks: FxHashSet<ChunkUkey>,
}

impl RemoveParentModulesContext {
  pub fn add_root_chunk(&mut self, ukey: ChunkUkey) {
    self.root_chunks.insert(ukey);
    self.chunk_relation_graph.add_node(ukey);
  }

  pub fn add_chunk_relation(&mut self, parent: ChunkUkey, child: ChunkUkey) {
    self.chunk_relation_graph.add_edge(parent, child, ());
  }
}

impl<'me> CodeSplitter<'me> {
  fn prepare_remove_parent_modules(&mut self) -> FxHashMap<ChunkUkey, DefinitelyLoadedModules> {
    #[derive(Debug, Clone)]
    struct AnalyzeContext<'a> {
      target_ukey: ChunkUkey,
      analyzing_chunks: FxHashSet<ChunkUkey>,
      root_chunks: &'a FxHashSet<ChunkUkey>,
      compilation: &'a Compilation,
      chunk_relation_graph: &'a ChunkRelationGraph,
    }

    impl<'a> AnalyzeContext<'a> {
      fn chunk_modules(&self, chunk: &ChunkUkey) -> &IdentifierSet {
        self
          .compilation
          .chunk_graph
          .get_chunk_module_identifiers(chunk)
      }
    }

    #[tracing::instrument(skip_all)]
    fn analyze_loaded_modules(mut ctx: AnalyzeContext) -> Option<DefinitelyLoadedModules> {
      if ctx.analyzing_chunks.contains(&ctx.target_ukey) {
        // We are in a circle.
        // For case: ChunkA[a.js, shared.js] <--> ChunkB[b.js, shared.js]
        // Just return a empty vec.
        None
      } else if ctx.root_chunks.contains(&ctx.target_ukey) {
        // we are in a root chunk.
        // Just return the modules itself.
        Some(ctx.chunk_modules(&ctx.target_ukey).clone())
      } else {
        ctx.analyzing_chunks.insert(ctx.target_ukey);
        let loaded_modules = ctx
          .chunk_relation_graph
          .edges_directed(ctx.target_ukey, petgraph::Direction::Incoming)
          .par_bridge()
          .map(|(parent, _me, _)| parent)
          .flat_map(|parent| {
            analyze_loaded_modules(AnalyzeContext {
              target_ukey: parent,
              ..ctx.clone()
            })
          })
          .fold(IdentifierSet::default, |mut acc, cur| {
            // The word `Definitely` in `DefinitelyLoadedModules` infers that
            // we need the intersection of all parent loaded modules.
            acc.retain(|x| cur.contains(x));
            acc
          })
          .flatten()
          // With the module itself
          .chain(ctx.chunk_modules(&ctx.target_ukey).par_iter().cloned())
          .collect::<IdentifierSet>();

        Some(loaded_modules)
      }
    }

    self
      .compilation
      .chunk_by_ukey
      .values()
      .par_bridge()
      .map(|chunk| {
        let loaded_modules = analyze_loaded_modules(AnalyzeContext {
          target_ukey: chunk.ukey,
          analyzing_chunks: FxHashSet::default(),
          root_chunks: &self.remove_parent_modules_context.root_chunks,
          compilation: self.compilation,
          chunk_relation_graph: &self.remove_parent_modules_context.chunk_relation_graph,
        })
        .unwrap_or_default();

        (chunk.ukey, loaded_modules)
      })
      .collect()
  }

  #[tracing::instrument(skip_all)]
  fn analyze_modules_should_be_removed(
    &mut self,
    loaded_modules_map: FxHashMap<ChunkUkey, DefinitelyLoadedModules>,
  ) -> Vec<(ChunkUkey, ModuleIdentifier)> {
    self
      .compilation
      .chunk_by_ukey
      .values()
      .par_bridge()
      .filter(|chunk| {
        // Fast path and correctness: We only need to analyze the chunk which is not a root chunk.
        !self
          .remove_parent_modules_context
          .root_chunks
          .contains(&chunk.ukey)
      })
      .flat_map(|chunk| {
        let parents_loaded_modules = Arc::new(
          self
            .remove_parent_modules_context
            .chunk_relation_graph
            .edges_directed(chunk.ukey, petgraph::Direction::Incoming)
            .map(|(parent, _me, _)| parent)
            .map(|parent| {
              loaded_modules_map
                .get(&parent)
                .expect("loaded_modules not found")
            })
            .collect::<Vec<_>>(),
        );
        self
          .compilation
          .chunk_graph
          .get_chunk_module_identifiers(&chunk.ukey)
          .into_par_iter()
          .cloned()
          .flat_map(move |module| {
            let is_all_parents_load_this_module = parents_loaded_modules
              .clone()
              .iter()
              .all(|loaded_modules| loaded_modules.contains(&module));

            if is_all_parents_load_this_module {
              Some((chunk.ukey, module))
            } else {
              None
            }
          })
      })
      .collect::<Vec<_>>()
  }

  #[tracing::instrument(skip_all)]
  fn remove_modules(&mut self, modules_should_be_removed: Vec<(ChunkUkey, ModuleIdentifier)>) {
    for (chunk_ukey, module) in modules_should_be_removed {
      self
        .compilation
        .chunk_graph
        .disconnect_chunk_and_module(&chunk_ukey, module);
    }
  }

  /// TODO: The current implementation has a lot of repeated computing.
  #[tracing::instrument(skip_all)]
  pub(super) fn remove_parent_modules(&mut self) {
    println!("@@@remove_parent_modules");
    let loaded_modules_map = self.prepare_remove_parent_modules();
    let modules_should_be_removed = self.analyze_modules_should_be_removed(loaded_modules_map);
    self.remove_modules(modules_should_be_removed)
  }
}
