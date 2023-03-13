use rspack_core::{ChunkUkey, Plugin};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::id_helpers::{
  assign_ascending_chunk_ids, get_short_module_name, request_to_id, shorten_long_string,
};

#[derive(Debug)]
pub struct StableNamedChunkIdsPlugin {
  pub delimiter: String,
  pub context: Option<String>,
}

impl StableNamedChunkIdsPlugin {
  pub fn new(delimiter: Option<String>, context: Option<String>) -> Self {
    Self {
      delimiter: delimiter.unwrap_or_else(|| "~".to_string()),
      context,
    }
  }
}

impl Plugin for StableNamedChunkIdsPlugin {
  fn chunk_ids(&mut self, compilation: &mut rspack_core::Compilation) -> rspack_error::Result<()> {
    let context = self
      .context
      .clone()
      .unwrap_or_else(|| compilation.options.context.to_string_lossy().to_string());

    // If the chunk has a name, use it as the id

    let mut used_ids: FxHashSet<String> = FxHashSet::default();

    let dynamic_imported = compilation
      .chunk_graph
      .split_point_module_identifier_to_chunk_ukey
      .iter()
      .map(|(k, v)| (v, k))
      .collect::<FxHashMap<_, _>>();

    compilation.chunk_by_ukey.values_mut().for_each(|chunk| {
      if let Some(name) = &chunk.name {
        chunk.id = Some(name.clone());
        chunk.ids = vec![name.clone()];
      } else if let Some(m) = dynamic_imported.get(&chunk.ukey) {
        let module = compilation
          .module_graph
          .module_by_identifier(m)
          .expect("Module should exist");
        let name = request_to_id(&get_short_module_name(module, &context));
        if used_ids.contains(&name) {
          panic!("Duplicate chunk id: {}", chunk.chunk_reasons.join("_"))
        } else {
          used_ids.insert(name.clone());
        }
        chunk.id = Some(name.clone());
        chunk.ids = vec![name];
      }
    });

    // Filter out chunks that don't have an id
    let chunks = compilation
      .chunk_by_ukey
      .values()
      .filter(|chunk| chunk.id.is_none())
      .collect::<Vec<_>>();

    let mut name_to_chunks: FxHashMap<String, FxHashSet<ChunkUkey>> = Default::default();

    let mut unnamed_chunks = vec![];

    chunks.iter().cloned().for_each(|chunk| {
      // If the chunk has no name, the chunk is mostly created by bundle splitting;
      let mut chunk_names_in_same_group = chunk
        .groups
        .iter()
        .map(|group| {
          compilation
            .chunk_group_by_ukey
            .get(group)
            .expect("Must have a ChunkGroup")
        })
        .flat_map(|group| group.chunks.iter())
        .filter(|each_chunk| **each_chunk != chunk.ukey)
        .map(|chunk| chunk.as_ref(&compilation.chunk_by_ukey))
        .filter_map(|c| c.id.as_deref())
        .collect::<Vec<_>>();
      chunk_names_in_same_group.sort_unstable();

      if chunk_names_in_same_group.len() == 1 {
        // All modules of the chunk is from one group, to avoid conflict, we need to add a suffix
        chunk_names_in_same_group.insert(0, "split");
      }

      let name = shorten_long_string(
        chunk_names_in_same_group.join(&self.delimiter),
        &self.delimiter,
      );

      if name.is_empty() {
        unnamed_chunks.push(chunk.ukey);
      } else {
        name_to_chunks.entry(name).or_default().insert(chunk.ukey);
      }
    });

    name_to_chunks.into_iter().for_each(|(name, chunks)| {
      if chunks.len() == 1 {
        let chunk = chunks.into_iter().next().expect("Must have a chunk");
        chunk.as_mut(&mut compilation.chunk_by_ukey).id = Some(name.clone());
        chunk.as_mut(&mut compilation.chunk_by_ukey).ids.push(name);
      } else {
        // Multiple chunks have the same name, we need to assign a unique name to each chunk
        let mut chunks = chunks.into_iter().collect::<Vec<_>>();
        chunks.sort_unstable_by(|a, b| {
          // TODO: We might need use more filed in sorting
          let a_module_ids = compilation.chunk_graph.get_chunk_module_identifiers(a);
          let b_module_ids = compilation.chunk_graph.get_chunk_module_identifiers(b);
          a_module_ids.len().cmp(&b_module_ids.len())
        });
        chunks.iter().enumerate().for_each(|(index, chunk)| {
          let chunk = chunk.as_mut(&mut compilation.chunk_by_ukey);
          let name = if index == 0 {
            name.to_string()
          } else {
            format!("{name}{index}")
          };
          chunk.id = Some(name.clone());
          chunk.ids.push(name);
        });
      }
    });

    if !unnamed_chunks.is_empty() {
      assign_ascending_chunk_ids(&unnamed_chunks, compilation)
    }

    Ok(())
  }
}
