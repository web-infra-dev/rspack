use std::collections::HashMap;

use rspack_core::Plugin;

use crate::id_helpers::{
  assign_deterministic_ids, compare_chunks_natural, get_full_chunk_name, get_used_chunk_ids,
};

#[derive(Debug, Default)]
pub struct DeterministicChunkIdsPlugin {
  pub delimiter: String,
  pub context: Option<String>,
}

impl DeterministicChunkIdsPlugin {
  pub fn new(delimiter: Option<String>, context: Option<String>) -> Self {
    Self {
      delimiter: delimiter.unwrap_or_else(|| "~".to_string()),
      context,
    }
  }
}

impl Plugin for DeterministicChunkIdsPlugin {
  fn chunk_ids(&self, compilation: &mut rspack_core::Compilation) -> rspack_error::Result<()> {
    let mut used_ids = get_used_chunk_ids(compilation);
    let used_ids_len = used_ids.len();

    let chunk_graph = &compilation.chunk_graph;
    let module_graph = compilation.get_module_graph();
    let context = self
      .context
      .clone()
      .unwrap_or_else(|| compilation.options.context.as_str().to_string());

    let max_length = 3;
    let expand_factor = 10;
    let salt = 10;

    let chunks = compilation
      .chunk_by_ukey
      .values()
      .filter(|chunk| chunk.id.is_none())
      .collect::<Vec<_>>();
    let mut chunk_key_to_id = HashMap::with_capacity(chunks.len());

    assign_deterministic_ids(
      chunks,
      |chunk| get_full_chunk_name(chunk, chunk_graph, &module_graph, &context),
      |a, b| compare_chunks_natural(chunk_graph, &module_graph, a, b),
      |chunk, id| {
        let size = used_ids.len();
        used_ids.insert(id.to_string());
        if used_ids.len() == size {
          return false;
        }

        chunk_key_to_id.insert(chunk.ukey, id);
        true
      },
      &[usize::pow(10, max_length)],
      expand_factor,
      used_ids_len,
      salt,
    );

    chunk_key_to_id.into_iter().for_each(|(chunk_ukey, id)| {
      let chunk = compilation.chunk_by_ukey.expect_get_mut(&chunk_ukey);
      chunk.id = Some(id.to_string());
      chunk.ids = vec![id.to_string()];
    });

    Ok(())
  }
}
