use std::sync::Arc;

use rspack_collections::{IdentifierSet, UkeyMap, UkeySet};
use rspack_core::{
  ChunkUkey, Compilation, CompilationOptimizeChunks, ModuleIdentifier, Plugin,
  incremental::Mutation,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rustc_hash::FxHashMap;

/**
entryA: a b
entryB: a b c
entryC: a b c d
we should extract them into ab, c, d

chunkMap:
a: [entryA, entryB, entryC]
b: [entryA, entryB, entryC]
c: [entryB, entryC]
d: [entryC]

according to this relations, we should make chunks for modules with same entries

a and b has same entry combinations
c has unique entry combinations
d has unique entry combinations
so we should create separate chunks for them, which is a+b, c, d
*/
#[derive(Debug)]
#[plugin]
pub struct RemoveDuplicateModulesPlugin {}

impl std::default::Default for RemoveDuplicateModulesPlugin {
  fn default() -> Self {
    Self {
      inner: Arc::new(RemoveDuplicateModulesPluginInner {}),
    }
  }
}

// if merge a chunk will break the signatures, we should not merge it
// only entry chunks have signature requirements
fn can_merge_chunk(chunk_modules: &IdentifierSet, entry_modules: &IdentifierSet) -> bool {
  if entry_modules.is_empty() {
    // normal chunk can always be merged as they don't have signature requirement
    return true;
  }

  // this chunk has entry module
  if chunk_modules.is_empty() || chunk_modules.len() > entry_modules.len() {
    return false;
  }

  // should be exact same entries
  chunk_modules.iter().all(|m| entry_modules.contains(m))
}

#[plugin_hook(CompilationOptimizeChunks for RemoveDuplicateModulesPlugin, stage = Compilation::OPTIMIZE_CHUNKS_STAGE_BASIC)]
async fn optimize_chunks(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  let module_graph = compilation.get_module_graph();
  let chunk_graph = &compilation.chunk_graph;

  let mut chunk_map: FxHashMap<Vec<ChunkUkey>, Vec<ModuleIdentifier>> = FxHashMap::default();
  let mut chunk_deps: UkeyMap<ChunkUkey, UkeySet<ChunkUkey>> = UkeyMap::default();
  let mut entry_modules_for_chunk: UkeyMap<ChunkUkey, IdentifierSet> = UkeyMap::default();
  let mut new_chunks: UkeySet<ChunkUkey> = UkeySet::default();

  for identifier in module_graph.modules().keys() {
    let chunks = chunk_graph.get_module_chunks(*identifier);
    let mut sorted_chunks = chunks.iter().copied().collect::<Vec<_>>();
    sorted_chunks.sort_by(|a, b| {
      let a = compilation.chunk_by_ukey.expect_get(a);
      let b = compilation.chunk_by_ukey.expect_get(b);

      let a = compilation
        .chunk_group_by_ukey
        .expect_get(a.groups().iter().next().expect("should have chunk group"));
      let b = compilation
        .chunk_group_by_ukey
        .expect_get(b.groups().iter().next().expect("should have chunk group"));
      a.index.cmp(&b.index)
    });
    chunk_map
      .entry(sorted_chunks)
      .or_default()
      .push(*identifier);
  }

  // potential chunks

  for (chunks, modules) in chunk_map {
    if chunks.len() <= 1 {
      continue;
    }

    let new_chunk_ukey = Compilation::add_chunk(&mut compilation.chunk_by_ukey);
    if let Some(mut mutations) = compilation.incremental.mutations_write() {
      mutations.add(Mutation::ChunkAdd {
        chunk: new_chunk_ukey,
      });
    }

    let new_chunk = compilation.chunk_by_ukey.expect_get_mut(&new_chunk_ukey);
    *new_chunk.chunk_reason_mut() = Some("modules are shared across multiple chunks".into());

    compilation.chunk_graph.add_chunk(new_chunk_ukey);

    for chunk in &chunks {
      chunk_deps.entry(*chunk).or_default().insert(new_chunk_ukey);
    }

    // split chunks from original chunks and create new chunk
    let mut entry_modules = IdentifierSet::default();

    for chunk_ukey in &chunks {
      let [Some(new_chunk), Some(origin)] = compilation
        .chunk_by_ukey
        .get_many_mut([&new_chunk_ukey, chunk_ukey])
      else {
        panic!("should have both chunks")
      };

      let cur_entry_modules = compilation.chunk_graph.get_chunk_entry_modules(chunk_ukey);
      entry_modules_for_chunk
        .entry(*chunk_ukey)
        .or_default()
        .extend(cur_entry_modules.clone().into_iter());

      entry_modules.extend(cur_entry_modules);
      origin.split(new_chunk, &mut compilation.chunk_group_by_ukey);
      if let Some(mut mutations) = compilation.incremental.mutations_write() {
        mutations.add(Mutation::ChunkSplit {
          from: *chunk_ukey,
          to: new_chunk_ukey,
        });
      }
      new_chunks.insert(new_chunk_ukey);
    }

    for m in modules {
      let is_entry = entry_modules.contains(&m);
      for chunk_ukey in &chunks {
        compilation
          .chunk_graph
          .disconnect_chunk_and_module(chunk_ukey, m);

        if is_entry {
          compilation
            .chunk_graph
            .disconnect_chunk_and_entry_module(chunk_ukey, m);
        }
      }

      compilation
        .chunk_graph
        .connect_chunk_and_module(new_chunk_ukey, m);

      if is_entry {
        let chunk = compilation.chunk_by_ukey.expect_get(&new_chunk_ukey);
        for group in chunk.groups().iter().filter(|group| {
          let group = compilation.chunk_group_by_ukey.expect_get(group);

          group.is_initial() && group.kind.is_entrypoint()
        }) {
          compilation
            .chunk_graph
            .connect_chunk_and_entry_module(new_chunk_ukey, m, *group);
        }
      }
    }
  }

  // try reusing existing chunks
  // for example, if we have 2 entries:
  // entryA: a b
  // entryB: a b c
  // after above process, we have 3 chunks:
  // newChunk: [a, b]
  // entryA:   [] + chunk
  // entryB:   [c] + chunk
  // you can see that chunk "a b" is exactly same as entryA, so we can reuse entryA chunk directly
  // and make it like:
  // entryA: [a, b]
  // entryB: [c] + entryA
  // first we find all empty chunks
  let empty_set = Default::default();
  let chunks = compilation
    .chunk_by_ukey
    .keys()
    .copied()
    .collect::<Vec<_>>();

  for chunk_ukey in chunks {
    if !compilation
      .chunk_graph
      .get_chunk_modules_identifier(&chunk_ukey)
      .is_empty()
      || !chunk_deps.contains_key(&chunk_ukey)
    {
      // chunk modules is not empty, or no dependent chunks, no need to merge
      continue;
    }

    if chunk_deps[&chunk_ukey].len() != 1 {
      continue;
    }

    // this is an empty chunk, and it only depends on 1 chunk, try reusing the existing chunks
    let dep_chunk = chunk_deps[&chunk_ukey]
      .iter()
      .next()
      .copied()
      .expect("already checked len == 1");

    let chunk_modules = compilation
      .chunk_graph
      .get_chunk_modules_identifier(&dep_chunk)
      .clone();
    let entry_modules = entry_modules_for_chunk
      .get(&chunk_ukey)
      .unwrap_or(&empty_set);
    if !can_merge_chunk(&chunk_modules, entry_modules) {
      // it's not safe to reuse chunk for chunks that contains entry modules
      // eg. entryA: [a, b] and import shared module
      //     entryB: [a] and import shared module
      // after splitting, we have 3 chunks:
      //     chunk1: [b, shared]
      //     entryA: [a] + chunk1
      //     entryB: [] + chunk1
      // how ever we know that b is an entrypoint, it needs to keep its signature,
      // it cannot exported exports from shared module, so we cannot reuse chunk1 for entryB
      continue;
    }

    // reuse target chunk
    let chunk = compilation.chunk_by_ukey.expect_get(&chunk_ukey);
    let group = chunk
      .groups()
      .iter()
      .find(|group| {
        let group = compilation.chunk_group_by_ukey.expect_get(group);
        group.is_initial() && group.kind.is_entrypoint()
      })
      .copied();

    for m in chunk_modules.iter().copied() {
      compilation
        .chunk_graph
        .disconnect_chunk_and_module(&dep_chunk, m);

      compilation
        .chunk_graph
        .connect_chunk_and_module(chunk_ukey, m);

      if entry_modules.contains(&m) {
        compilation
          .chunk_graph
          .disconnect_chunk_and_entry_module(&dep_chunk, m);

        compilation.chunk_graph.connect_chunk_and_entry_module(
          chunk_ukey,
          m,
          group.expect("should have entrypoint for chunk contains entry modules"),
        );
      }
    }

    // remove the pointed chunk
    let groups = compilation
      .chunk_by_ukey
      .expect_get(&dep_chunk)
      .groups()
      .clone();
    for group in groups {
      let group = compilation.chunk_group_by_ukey.expect_get_mut(&group);
      group.remove_chunk(&dep_chunk);
    }

    compilation.chunk_graph.remove_chunk(&dep_chunk);
    compilation.chunk_by_ukey.remove(&dep_chunk);
    new_chunks.remove(&dep_chunk);
  }

  if let Some(mut mutations) = compilation.incremental.mutations_write() {
    for chunk in new_chunks {
      mutations.add(Mutation::ChunkAdd { chunk });
    }
  }

  Ok(None)
}

impl Plugin for RemoveDuplicateModulesPlugin {
  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .compilation_hooks
      .optimize_chunks
      .tap(optimize_chunks::new(self));
    Ok(())
  }
}
