use std::{collections::HashSet, fmt::write, path::PathBuf};

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
    // generate following chunk_group_info as dto record info
    // <td title="chunk_group_name"></td><td title="chunk1"></td><td title="chunk2"></td>
    let get_debug_chunk_group_info = |chunk_group_ukey: &ChunkGroupUkey| {
      let chunk_group = compilation
        .chunk_group_by_ukey
        .get(&chunk_group_ukey)
        .expect("should have chunk group");
      let chunk_group_name_id = chunk_group_ukey.as_u32().to_string();
      let chunk_group_name = chunk_group.name().unwrap_or(chunk_group_name_id.as_str());
      let table_header = format!("<tr><td bgcolor=\"#aaa\">{}</td></tr>", chunk_group_name);
      let bg_color = if chunk_group.is_initial() {
        "green"
      } else {
        "orange"
      };

      let requests = chunk_group
        .chunks
        .iter()
        .filter_map(|chunk_ukey| {
          let chunk: &crate::Chunk = compilation
            .chunk_by_ukey
            .get(chunk_ukey)
            .expect("should have chunk");
          // let mg = &compilation.get_module_graph();
          // let mg = self.get_ordered_chunk_modules(chunk_ukey, mg);
          // let modules = mg.iter().map(|m| m.identifier()).collect::<Vec<_>>();
          if let Some(name) = chunk.name() {
            return Some(name.to_string());
          }
          let id = chunk_ukey.as_u32().to_string();
          return Some(id);
        })
        .map(|chunk_name| format!("    <tr><td>{}</td></tr>", chunk_name))
        .join("\n");

      let table_body = format!("{}", requests);

      return format!(
        "\n<<table bgcolor=\"{}\">\n{}\n{}\n</table>>\n",
        bg_color, table_header, table_body
      );
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
    write!(&mut dot, "node [shape=plaintext];\n")?;

    // write all node info
    for (node_id, node_info) in visited_group_nodes.iter() {
      write!(&mut dot, "{} {} [\n", INDENT, node_id.as_u32())?;
      write!(&mut dot, "label={}", node_info)?;
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
    let result = String::from_utf8_lossy(&dot).to_string();

    return Ok(result);
  }
  pub fn generate_dot(&self, compilation: &Compilation, dotfile_name: &str) -> std::io::Result<()> {
    let result = self.to_dot(compilation)?;
    std::fs::write(
      format!(
        "{}-{}.dot",
        compilation.compiler_id().as_u32(),
        dotfile_name
      ),
      &result,
    )?;
    Ok(())
  }
}
