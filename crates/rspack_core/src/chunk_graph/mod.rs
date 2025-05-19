use std::{collections::HashSet, fmt::write};

use itertools::Itertools;
use rspack_collections::{IdentifierMap, UkeyMap};
use rustc_hash::FxHashMap as HashMap;

use crate::{
  chunk, chunk_group, AsyncDependenciesBlockIdentifier, ChunkGroupUkey, ChunkUkey, Compilation,
  ModuleIdentifier,
};

pub mod chunk_graph_chunk;
pub mod chunk_graph_module;
pub use chunk_graph_chunk::{ChunkGraphChunk, ChunkSizeOptions};
pub use chunk_graph_module::{ChunkGraphModule, ModuleId};

#[derive(Debug, Clone, Default)]
pub struct ChunkGraph {
  /// If a module is imported dynamically, it will be assigned to a unique ChunkGroup
  pub(crate) block_to_chunk_group_ukey: HashMap<AsyncDependenciesBlockIdentifier, ChunkGroupUkey>,

  pub(crate) chunk_graph_module_by_module_identifier: IdentifierMap<ChunkGraphModule>,
  chunk_graph_chunk_by_chunk_ukey: UkeyMap<ChunkUkey, ChunkGraphChunk>,

  runtime_ids: HashMap<String, Option<String>>,
}

impl ChunkGraph {
  pub fn is_entry_module(&self, module_id: &ModuleIdentifier) -> bool {
    let cgm = self.expect_chunk_graph_module(*module_id);
    !cgm.entry_in_chunks.is_empty()
  }
}
static INDENT: &str = "    ";

impl ChunkGraph {
  // convert chunk graph to dot format
  // 1. support chunk_group dump visualizer
  pub fn to_dot(&self, compilation: &Compilation) -> std::io::Result<String> {
    let mut visited_group_nodes: HashMap<ChunkGroupUkey, String> = HashMap::default();
    let mut visited_group_edges: HashSet<(ChunkGroupUkey, ChunkGroupUkey)> = HashSet::new();
    let mut visiting_groups: Vec<ChunkGroupUkey> = Vec::new();
    // format chunks into dot record
    let get_debug_chunk_group_info = |chunk_group_ukey: &ChunkGroupUkey| {
      let chunk_group = compilation
        .chunk_group_by_ukey
        .get(&chunk_group_ukey)
        .expect("should have chunk group");
      let name_id = chunk_group_ukey.as_u32().to_string();
      let name = chunk_group.name().unwrap_or(name_id.as_str());

      let requests = chunk_group
        .origins()
        .iter()
        .filter_map(|x| x.request.as_ref().map(|x| x.as_str()));

      let request = std::iter::once(name).chain(requests).join(" | ");
      return request;
    };

    // push entry_point chunk group into visiting queue
    for (_, chunk_group_ukey) in compilation.entrypoints() {
      visiting_groups.push(*chunk_group_ukey);
    }
    // bfs visit all chunk groups
    while let Some(chunk_group_ukey) = visiting_groups.pop() {
      if visited_group_nodes.contains_key(&chunk_group_ukey) {
        continue;
      }
      let chunk_group_name = get_debug_chunk_group_info(&chunk_group_ukey);
      visited_group_nodes.insert(chunk_group_ukey, chunk_group_name.clone());
      let chunk_group = compilation
        .chunk_group_by_ukey
        .get(&chunk_group_ukey)
        .expect("should have chunk group");
      for child in chunk_group.children.iter() {
        let chunk_group_name = get_debug_chunk_group_info(&child);
        // calculate every edge
        visited_group_edges.insert((chunk_group_ukey, *child));
        visited_group_nodes.insert(*child, chunk_group_name);
        visiting_groups.push(*child);
      }
    }
    use std::io::Write;
    let mut dot = Vec::new();
    // write header
    write!(&mut dot, "digraph G {{\n")?;
    // write all node info
    // 1 [ label = "1" info = "a | b | c" shape = record ]
    for (node_id, node_info) in visited_group_nodes.iter() {
      write!(&mut dot, "{} {} [\n", INDENT, node_id.as_u32())?;
      write!(&mut dot, "label=\"{}\"", node_info)?;
      write!(&mut dot, "\nshape=record")?;
      write!(&mut dot, "\n];\n")?;
    }
    // write all edge info
    // 1 -> 2, 2 -> 3
    for edge in visited_group_edges.iter() {
      write!(&mut dot, "{} -> {}", edge.0.as_u32(), edge.1.as_u32())?;
      write!(&mut dot, ";\n")?;
    }
    // write footer
    write!(&mut dot, "}}")?;
    return Ok(String::from_utf8_lossy(&dot).to_string());
  }
}
