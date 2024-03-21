use dashmap::DashMap;
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
  fn chunk_ids(&self, compilation: &mut rspack_core::Compilation) -> rspack_error::Result<()> {
    use rayon::prelude::*;
    // code_splitting_chunk means chunks generated in code splitting
    let code_splitting_chunk_to_root_module = compilation
      .chunk_graph
      .split_point_module_identifier_to_chunk_ukey
      .iter()
      .map(|(k, v)| (v, k))
      .collect::<FxHashMap<_, _>>();

    let context = self
      .context
      .clone()
      .unwrap_or_else(|| compilation.options.context.to_string());

    let mut used_code_splitting_chunk_name: FxHashSet<String> = FxHashSet::default();

    let mut chunk_by_ukey = std::mem::take(&mut compilation.chunk_by_ukey);
    let chunks_to_be_named = chunk_by_ukey
      .values_mut()
      .filter_map(|chunk| {
        // If the chunk has a name already, use it as the id
        if let Some(name) = &chunk.name {
          // Initial chunk create in Code Splitting definitely has a name
          chunk.id = Some(name.clone());
          chunk.ids = vec![name.clone()];
        } else if let Some(root_id) = code_splitting_chunk_to_root_module.get(&chunk.ukey) {
          // Make sure all chunks create in code splitting must have a name
          let root_module = compilation
            .get_module_graph()
            .module_by_identifier(root_id)
            .expect("Module should exist");
          let root_module_name = request_to_id(&get_short_module_name(root_module, &context));

          let name = chunk
            .id_name_hints
            .iter()
            .map(|s| s.as_str())
            .chain([root_module_name.as_str()])
            .intersperse(self.delimiter.as_str())
            .collect::<String>();

          if used_code_splitting_chunk_name.contains(&name) {
            // This is not likely to happen, any module could not be the root of multiple chunks
            panic!(
              "Duplicate root module name {}",
              chunk.chunk_reasons.join("_")
            )
          } else {
            used_code_splitting_chunk_name.insert(name.clone());
          }

          chunk.id = Some(name.clone());
          chunk.ids = vec![name];
        }

        if chunk.id.is_none() {
          Some(chunk.ukey)
        } else {
          None
        }
      })
      .collect::<Vec<_>>();
    compilation.chunk_by_ukey = chunk_by_ukey;
    let name_to_chunks: DashMap<String, FxHashSet<ChunkUkey>> = Default::default();

    let chunks_has_no_name = chunks_to_be_named
      .into_par_iter()
      .filter_map(|chunk| {
        let chunk = chunk.as_ref(&compilation.chunk_by_ukey);

        // filter out chunks generated in code splitting in the same `ChunkGroup`
        let code_splitting_chunks = chunk
          .groups
          .iter()
          .flat_map(|group| group.as_ref(&compilation.chunk_group_by_ukey).chunks.iter())
          .filter(|chunk| code_splitting_chunk_to_root_module.contains_key(chunk))
          .map(|c| c.as_ref(&compilation.chunk_by_ukey));

        let mut code_splitting_chunk_names = code_splitting_chunks
          .map(|c| {
            c.id.as_deref().unwrap_or_else(|| {
              panic!(
                "CodeSplitting chunks must have a name {:?}",
                chunk.chunk_reasons.join("_")
              )
            })
          })
          .collect::<Box<[&str]>>();

        code_splitting_chunk_names.sort_unstable();

        let name = if code_splitting_chunk_names.len() == 1 {
          // All modules of this chunk is from single code splitting chunk. To reduce naming conflicts,
          // we need to add a prefix 'splitting'
          chunk
            .id_name_hints
            .iter()
            .map(|s| s.as_str())
            .chain(["splitting"])
            .chain(code_splitting_chunk_names.iter().copied())
            .intersperse(&self.delimiter)
            .collect::<String>()
        } else {
          chunk
            .id_name_hints
            .iter()
            .map(|s| s.as_str())
            .chain(code_splitting_chunk_names.iter().copied())
            .intersperse(&self.delimiter)
            .collect::<String>()
        };

        let name = shorten_long_string(name, &self.delimiter);

        if name.is_empty() {
          Some(chunk.ukey)
        } else {
          name_to_chunks.entry(name).or_default().insert(chunk.ukey);
          None
        }
      })
      .collect::<Vec<_>>();

    name_to_chunks.into_iter().for_each(|(name, chunks)| {
      if chunks.len() == 1 {
        // The name has no conflicts to other chunks, we just use it as the chunk id
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

    if !chunks_has_no_name.is_empty() {
      assign_ascending_chunk_ids(&chunks_has_no_name, compilation)
    }

    Ok(())
  }
}
