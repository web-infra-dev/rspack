use std::fmt::Write;

use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext,
  RuntimeModuleRuntimeRequirements, RuntimeTemplate, impl_runtime_module,
};

// Prefix lengths use JavaScript UTF-16 units; suffix boundaries stay on Rust chars.
// Decoding reconstructs the exact existing identity, without hashing or allocation IDs.
fn front_code<'a>(previous: &str, value: &'a str) -> (usize, &'a str) {
  let mut prefix_utf16 = 0;
  let mut prefix_bytes = 0;
  for (left, right) in previous.chars().zip(value.chars()) {
    if left != right {
      break;
    }
    prefix_utf16 += right.len_utf16();
    prefix_bytes += right.len_utf8();
  }
  (prefix_utf16, &value[prefix_bytes..])
}

#[impl_runtime_module]
#[derive(Debug)]
pub struct EnsureAsyncBlockRuntimeModule {}

impl EnsureAsyncBlockRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(runtime_template)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for EnsureAsyncBlockRuntimeModule {
  fn runtime_module_variables() -> &'static [&'static str] {
    &[]
  }

  fn runtime_requirements(&self, _compilation: &Compilation) -> RuntimeModuleRuntimeRequirements {
    RuntimeModuleRuntimeRequirements {
      define: RuntimeGlobals::ENSURE_ASYNC_BLOCK,
      ..Default::default()
    }
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let compilation = context.compilation;
    let graph = &compilation.build_chunk_graph_artifact;
    let runtime_chunk = graph
      .chunk_by_ukey
      .expect_get(&self.chunk().expect("runtime module attached"));
    let referenced_chunks = runtime_chunk.get_all_referenced_chunks(&graph.chunk_group_by_ukey);
    let runtime_modules = referenced_chunks
      .iter()
      .flat_map(|chunk| graph.chunk_graph.get_chunk_modules_identifier(chunk))
      .copied()
      .collect::<rspack_collections::IdentifierSet>();
    let module_graph = compilation.get_module_graph();
    let mut blocks = Vec::new();
    let mut chunk_ids = Vec::new();

    for (block_id, block) in module_graph.blocks() {
      if !runtime_modules.contains(block.parent()) {
        continue;
      }
      let group = graph
        .chunk_graph
        .get_block_chunk_group(block_id, &graph.chunk_group_by_ukey);
      blocks.push(block_id);
      if let Some(group) = group {
        for chunk in &group.chunks {
          let chunk = graph.chunk_by_ukey.expect_get(chunk);
          if !chunk.has_runtime(&graph.chunk_group_by_ukey)
            && let Some(chunk_id) = chunk.id()
          {
            chunk_ids.push(chunk_id.as_str());
          }
        }
      }
    }
    blocks.sort_unstable_by_key(|block_id| block_id.as_str());

    // The map belongs to this runtime generation. All JS/CSS loading and fetch
    // priority behavior stays in the existing caller-provided ensure function.
    chunk_ids.sort_unstable();
    chunk_ids.dedup();
    let mut chunks = String::from("[");
    let mut previous_chunk = "";
    for (index, chunk_id) in chunk_ids.iter().enumerate() {
      if index != 0 {
        chunks.push(',');
      }
      if let Some(number) = rspack_util::numeric_id_value(chunk_id) {
        write!(chunks, "{number}").expect("infallible write to String");
      } else {
        let (prefix, suffix) = front_code(previous_chunk, chunk_id);
        write!(
          chunks,
          "[{prefix},{}]",
          rspack_util::json_stringify_str(suffix)
        )
        .expect("infallible write to String");
        previous_chunk = chunk_id;
      }
    }
    chunks.push(']');
    let mut rows = String::from("[");
    let mut previous_block = "";
    for (index, block_id) in blocks.iter().enumerate() {
      let group = graph
        .chunk_graph
        .get_block_chunk_group(block_id, &graph.chunk_group_by_ukey);
      let block_id = block_id.as_str();
      if index != 0 {
        rows.push(',');
      }
      let (prefix, suffix) = front_code(previous_block, block_id);
      previous_block = block_id;
      write!(
        rows,
        "[{prefix},{},[",
        rspack_util::json_stringify_str(suffix)
      )
      .expect("infallible write to String");
      if let Some(group) = group {
        let mut first = true;
        for chunk in &group.chunks {
          let chunk = graph.chunk_by_ukey.expect_get(chunk);
          if chunk.has_runtime(&graph.chunk_group_by_ukey) {
            continue;
          }
          if let Some(chunk_id) = chunk.id() {
            if !first {
              rows.push(',');
            }
            first = false;
            let chunk_index = chunk_ids
              .binary_search(&chunk_id.as_str())
              .expect("chunk in dictionary");
            write!(rows, "{chunk_index}").expect("infallible write to String");
          }
        }
      }
      rows.push_str("],");
      if let Some(priority) = group
        .and_then(|group| group.kind.get_normal_options())
        .and_then(|options| options.fetch_priority)
      {
        rows.push_str(&rspack_util::json_stringify_str(&priority.to_string()));
      } else {
        rows.push_str("null");
      }
      rows.push(']');
    }
    rows.push(']');
    let definition = context
      .runtime_template
      .render_runtime_global_definition(&RuntimeGlobals::ENSURE_ASYNC_BLOCK);
    Ok(format!(
      r#"
{definition} = (function(chunks, rows) {{
  var previous = "";
  chunks = chunks.map(function(row) {{
    return typeof row === "number" ? row : (previous = previous.slice(0, row[0]) + row[1]);
  }});
  var blocks = Object.create(null);
  previous = "";
  rows.forEach(function(row) {{
    previous = previous.slice(0, row[0]) + row[1];
    blocks[previous] = [row[2], row[3]];
  }});
  var ensureBlock = function(blockId, ensureChunk) {{
    var block = blocks[blockId];
    if (!block) return Promise.reject(new Error("Missing async block " + blockId));
    return Promise.all(block[0].map(function(chunkId) {{
      return ensureChunk(chunks[chunkId], block[1] || undefined);
    }}));
  }};
  ensureBlock.has = function(blockId) {{ return blocks[blockId] !== undefined; }};
  return ensureBlock;
}})({chunks}, {rows});
"#
    ))
  }
}
