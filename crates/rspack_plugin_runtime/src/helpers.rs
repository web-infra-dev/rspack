use std::hash::Hash;

use anyhow::anyhow;
use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Chunk, ChunkGroupByUkey, ChunkGroupUkey, ChunkUkey, Compilation, PathData, RenderChunkArgs,
  RuntimeGlobals,
};
use rspack_error::Result;
use rspack_identifier::IdentifierLinkedMap;
use rspack_plugin_javascript::runtime::stringify_chunks_to_array;
use rustc_hash::FxHashSet as HashSet;
use xxhash_rust::xxh3::Xxh3;

pub fn update_hash_for_entry_startup(
  hasher: &mut Xxh3,
  compilation: &Compilation,
  entries: &IdentifierLinkedMap<ChunkGroupUkey>,
  chunk: &ChunkUkey,
) {
  for (module, entry) in entries {
    if let Some(module_id) = compilation
      .module_graph
      .module_graph_module_by_identifier(module)
      .map(|module| module.id(&compilation.chunk_graph))
    {
      module_id.hash(hasher);
    }

    if let Some(runtime_chunk) = compilation
      .chunk_group_by_ukey
      .get(entry)
      .map(|e| e.get_runtime_chunk())
    {
      for chunk_ukey in get_all_chunks(
        entry,
        chunk,
        Some(&runtime_chunk),
        &compilation.chunk_group_by_ukey,
      ) {
        if let Some(chunk) = compilation.chunk_by_ukey.get(&chunk_ukey) {
          chunk.id.hash(hasher);
        }
      }
    }
  }
}

pub fn get_all_chunks(
  entrypoint: &ChunkGroupUkey,
  exclude_chunk1: &ChunkUkey,
  exclude_chunk2: Option<&ChunkUkey>,
  chunk_group_by_ukey: &ChunkGroupByUkey,
) -> HashSet<ChunkUkey> {
  fn add_chunks(
    chunk_group_by_ukey: &ChunkGroupByUkey,
    chunks: &mut HashSet<ChunkUkey>,
    entrypoint_ukey: &ChunkGroupUkey,
    exclude_chunk1: &ChunkUkey,
    exclude_chunk2: Option<&ChunkUkey>,
    visit_chunk_groups: &mut HashSet<ChunkGroupUkey>,
  ) {
    if let Some(entrypoint) = chunk_group_by_ukey.get(entrypoint_ukey) {
      for chunk in &entrypoint.chunks {
        if chunk == exclude_chunk1 {
          continue;
        }
        if let Some(exclude_chunk2) = exclude_chunk2 {
          if chunk == exclude_chunk2 {
            continue;
          }
        }
        chunks.insert(*chunk);
      }

      for parent in entrypoint.parents_iterable() {
        if visit_chunk_groups.contains(parent) {
          continue;
        }
        visit_chunk_groups.insert(*parent);
        if let Some(chunk_group) = chunk_group_by_ukey.get(parent) {
          if chunk_group.is_initial() {
            add_chunks(
              chunk_group_by_ukey,
              chunks,
              &chunk_group.ukey,
              exclude_chunk1,
              exclude_chunk2,
              visit_chunk_groups,
            );
          }
        }
      }
    }
  }

  let mut chunks: HashSet<ChunkUkey> = HashSet::default();
  let mut visit_chunk_groups = HashSet::default();

  add_chunks(
    chunk_group_by_ukey,
    &mut chunks,
    entrypoint,
    exclude_chunk1,
    exclude_chunk2,
    &mut visit_chunk_groups,
  );

  chunks
}

pub fn get_runtime_chunk_output_name(args: &RenderChunkArgs) -> Result<String> {
  let entry_point = {
    let entry_points = args
      .compilation
      .chunk_graph
      .get_chunk_entry_modules_with_chunk_group_iterable(args.chunk_ukey);

    let (_, entry_point_ukey) = entry_points
      .iter()
      .next()
      .ok_or_else(|| anyhow!("should has entry point ukey"))?;

    args
      .compilation
      .chunk_group_by_ukey
      .get(entry_point_ukey)
      .ok_or_else(|| anyhow!("should has entry point"))?
  };

  let runtime_chunk = args
    .compilation
    .chunk_by_ukey
    .get(&entry_point.get_runtime_chunk())
    .ok_or_else(|| anyhow!("should has runtime chunk"))?;

  Ok(get_chunk_output_name(runtime_chunk, args.compilation))
}

pub fn generate_entry_startup(
  compilation: &Compilation,
  chunk: &ChunkUkey,
  entries: &IdentifierLinkedMap<ChunkGroupUkey>,
  passive: bool,
) -> BoxSource {
  let mut module_ids = vec![];
  let mut chunks_ids = HashSet::default();

  for (module, entry) in entries {
    if let Some(module_id) = compilation
      .module_graph
      .module_graph_module_by_identifier(module)
      .map(|module| module.id(&compilation.chunk_graph))
    {
      module_ids.push(module_id);
    }

    if let Some(runtime_chunk) = compilation
      .chunk_group_by_ukey
      .get(entry)
      .map(|e| e.get_runtime_chunk())
    {
      let chunks = get_all_chunks(
        entry,
        chunk,
        Some(&runtime_chunk),
        &compilation.chunk_group_by_ukey,
      );
      chunks_ids.extend(
        chunks
          .iter()
          .map(|chunk_ukey| {
            let chunk = compilation
              .chunk_by_ukey
              .get(chunk_ukey)
              .expect("Chunk not found");
            chunk.expect_id().to_string()
          })
          .collect::<HashSet<_>>(),
      );
    }
  }

  let mut source = String::default();
  source.push_str(&format!(
    "var __webpack_exec__ = function(moduleId) {{ return __webpack_require__({} = moduleId) }}\n",
    RuntimeGlobals::ENTRY_MODULE_ID
  ));

  let module_ids_code = &module_ids
    .iter()
    .map(|id| format!("__webpack_exec__('{id}')"))
    .collect::<Vec<_>>()
    .join(", ");
  if chunks_ids.is_empty() {
    source.push_str("var __webpack_exports__ = (");
    source.push_str(module_ids_code);
    source.push_str(");\n");
  } else {
    if !passive {
      source.push_str("var __webpack_exports__ = ");
    }
    source.push_str(&format!(
      "{}(0, {}, function() {{
        return {};
      }});\n",
      if passive {
        RuntimeGlobals::ON_CHUNKS_LOADED
      } else {
        RuntimeGlobals::STARTUP_ENTRYPOINT
      },
      stringify_chunks_to_array(&chunks_ids),
      module_ids_code
    ));
    if passive {
      source.push_str(&format!(
        "var __webpack_exports__ = {}();\n",
        RuntimeGlobals::ON_CHUNKS_LOADED
      ));
    }
  }

  RawSource::from(source).boxed()
}

pub fn get_relative_path(base_chunk_output_name: &str, other_chunk_output_name: &str) -> String {
  let mut base_chunk_output_name_arr = base_chunk_output_name.split('/').collect::<Vec<_>>();
  base_chunk_output_name_arr.pop();
  let mut other_chunk_output_name_arr = other_chunk_output_name.split('/').collect::<Vec<_>>();
  while !base_chunk_output_name_arr.is_empty() && !other_chunk_output_name_arr.is_empty() {
    if base_chunk_output_name_arr[0] == other_chunk_output_name_arr[0] {
      base_chunk_output_name_arr.remove(0);
      other_chunk_output_name_arr.remove(0);
    }
  }
  let path = if base_chunk_output_name_arr.is_empty() {
    "./".to_string()
  } else {
    "../".repeat(base_chunk_output_name_arr.len())
  };
  format!("{path}{}", other_chunk_output_name_arr.join("/"))
}

pub fn get_chunk_output_name(chunk: &Chunk, compilation: &Compilation) -> String {
  let hash = chunk.get_render_hash();
  compilation.get_path(
    &compilation.options.output.chunk_filename,
    PathData::default()
      .chunk(chunk)
      .content_hash(&hash)
      .hash(&hash),
  )
}
