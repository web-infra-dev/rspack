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

    for (block_id, block) in module_graph.blocks() {
      if !runtime_modules.contains(block.parent()) {
        continue;
      }
      let group = graph
        .chunk_graph
        .get_block_chunk_group(block_id, &graph.chunk_group_by_ukey);
      let chunks = group
        .into_iter()
        .flat_map(|group| &group.chunks)
        .map(|chunk| graph.chunk_by_ukey.expect_get(chunk))
        .filter(|chunk| !chunk.has_runtime(&graph.chunk_group_by_ukey))
        .filter_map(|chunk| chunk.id().cloned())
        .collect::<Vec<_>>();
      let priority = group
        .and_then(|group| group.kind.get_normal_options())
        .and_then(|options| options.fetch_priority)
        .map(|priority| priority.to_string());
      blocks.push((block_id.as_str(), (chunks, priority)));
    }
    blocks.sort_unstable_by_key(|(block_id, _)| *block_id);

    // The map belongs to this runtime generation. All JS/CSS loading and fetch
    // priority behavior stays in the existing caller-provided ensure function.
    let mut chunk_ids = blocks
      .iter()
      .flat_map(|(_, (chunks, _))| chunks.iter().cloned())
      .collect::<Vec<_>>();
    chunk_ids.sort_unstable();
    chunk_ids.dedup();
    let mut encoded_chunks = Vec::new();
    let mut previous_chunk = "";
    for chunk_id in &chunk_ids {
      let encoded = if let Some(number) = chunk_id.as_number() {
        serde_json::json!(number)
      } else {
        let (prefix, suffix) = front_code(previous_chunk, chunk_id.as_str());
        let encoded = serde_json::json!([prefix, suffix]);
        previous_chunk = chunk_id.as_str();
        encoded
      };
      encoded_chunks.push(encoded);
    }
    let mut previous_block = "";
    let encoded_blocks = blocks
      .iter()
      .map(|(block_id, (chunks, priority))| {
        let (prefix, suffix) = front_code(previous_block, block_id);
        previous_block = block_id;
        let chunks = chunks
          .iter()
          .map(|chunk| chunk_ids.binary_search(chunk).expect("chunk in dictionary"))
          .collect::<Vec<_>>();
        (prefix, suffix, chunks, priority)
      })
      .collect::<Vec<_>>();
    let chunks = rspack_util::json_stringify(&encoded_chunks);
    let blocks = rspack_util::json_stringify(&encoded_blocks);
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
}})({chunks}, {blocks});
"#
    ))
  }
}
