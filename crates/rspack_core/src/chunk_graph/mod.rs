use core::fmt;
use std::borrow::Cow;

use itertools::Itertools;
use rspack_collections::{IdentifierMap, UkeyMap};
use rspack_util::env::has_query;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
  AsyncDependenciesBlockIdentifier, ChunkGroupUkey, ChunkUkey, Compilation, ModuleIdentifier,
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
  pub fn to_dot(&self, compilation: &Compilation) -> std::result::Result<String, fmt::Error> {
    let mut visited_group_nodes: HashMap<ChunkGroupUkey, String> = HashMap::default();
    let mut visited_group_edges: HashSet<(ChunkGroupUkey, ChunkGroupUkey, bool)> =
      HashSet::default();
    let mut visiting_groups: Vec<ChunkGroupUkey> = Vec::new();
    let module_graph = compilation.get_module_graph();
    // generate following chunk_group_info as dto record info
    // <td title="chunk_group_name"></td><td title="chunk1"></td><td title="chunk2"></td>
    let get_debug_chunk_group_info = |chunk_group_ukey: &ChunkGroupUkey| {
      let chunk_group = compilation
        .chunk_group_by_ukey
        .get(chunk_group_ukey)
        .expect("should have chunk group");

      let chunk_group_name = chunk_group.name().map_or_else(
        || {
          let mut origins = chunk_group
            .origins()
            .iter()
            .filter_map(|record| {
              record.request.as_deref().and_then(|request| {
                record.module.as_ref().map(|module_id| {
                  (
                    module_graph
                      .module_by_identifier(module_id)
                      .expect("should have module")
                      .readable_identifier(&compilation.options.context),
                    request,
                  )
                })
              })
            })
            .map(|(module, request)| format!("{module} {request}"))
            .collect::<Vec<_>>();

          origins.sort();
          Cow::Owned(origins.join("\n"))
        },
        Cow::Borrowed,
      );
      let table_header = format!("<tr><td bgcolor=\"#aaa\">{chunk_group_name}</td></tr>");
      let bg_color = if chunk_group.is_initial() {
        "green"
      } else {
        "orange"
      };

      let requests = chunk_group
        .chunks
        .iter()
        .map(|chunk_ukey| {
          let chunk: &crate::Chunk = compilation
            .chunk_by_ukey
            .get(chunk_ukey)
            .expect("should have chunk");
          if let Some(name) = chunk.name() {
            return name.to_string();
          }

          chunk_ukey.as_u32().to_string()
        })
        .map(|chunk_name| format!("    <tr><td>{chunk_name}</td></tr>"))
        .join("\n");

      let table_body = requests.clone();

      format!(
        r#"
<<table bgcolor="{bg_color}">
{table_header}
{table_body}
</table>>
"#
      )
    };

    // push entry_point chunk group into visiting queue
    for (_, chunk_group_ukey) in compilation.entrypoints() {
      visiting_groups.push(*chunk_group_ukey);
    }
    // bfs visit all chunk groups
    while let Some(chunk_group_ukey) = visiting_groups.pop() {
      let chunk_group = compilation
        .chunk_group_by_ukey
        .get(&chunk_group_ukey)
        .expect("should have chunk group");
      if visited_group_nodes.contains_key(&chunk_group_ukey) {
        continue;
      }
      let chunk_group_name = get_debug_chunk_group_info(&chunk_group_ukey);

      for parent in &chunk_group.parents {
        // false means this is a revert link to parent
        visited_group_edges.insert((chunk_group_ukey, *parent, false));
      }
      for child in chunk_group.children.iter() {
        // calculate every edge
        visited_group_edges.insert((chunk_group_ukey, *child, true));
        visiting_groups.push(*child);
      }
      visited_group_nodes.insert(chunk_group_ukey, chunk_group_name.clone());
    }
    use std::fmt::Write;
    let mut dot = String::new();
    // write header
    writeln!(&mut dot, "digraph G {{")?;
    // neato layout engine is more readable
    writeln!(&mut dot, "layout=neato;")?;
    writeln!(&mut dot, "overlap=false;")?;
    writeln!(&mut dot, "node [shape=plaintext];")?;
    writeln!(&mut dot, "edge [arrowsize=0.5];")?;

    // write all node info
    for (node_id, node_info) in visited_group_nodes.iter() {
      writeln!(&mut dot, "{} {} [", INDENT, node_id.as_u32())?;
      write!(&mut dot, "label={node_info}")?;
      write!(&mut dot, "\n];\n")?;
    }
    // write all edge info
    // 1 -> 2, 2 -> 3
    for edge in visited_group_edges.iter() {
      write!(&mut dot, "{} -> {}", edge.0.as_u32(), edge.1.as_u32())?;
      write!(&mut dot, "[")?;
      write!(
        &mut dot,
        "style=\"{}\"",
        if edge.2 { "solid" } else { "dotted" }
      )?;
      write!(&mut dot, "]")?;
      writeln!(&mut dot, ";")?;
    }
    // write footer
    write!(&mut dot, "}}")?;
    Ok(dot)
  }
  pub async fn generate_dot(&self, compilation: &Compilation, dotfile_name: &str) {
    // do not generate dot file if there is no query
    if !has_query() {
      return;
    }
    let result = self.to_dot(compilation).expect("to_dot failed");
    compilation
      .output_filesystem
      .write(
        format!(
          "{}-{}.dot",
          compilation.compiler_id().as_u32(),
          dotfile_name
        )
        .as_str()
        .into(),
        result.as_bytes(),
      )
      .await
      .expect("write dot file failed");
  }
}
