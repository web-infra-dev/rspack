use std::hash::Hash;

use rspack_collections::{IdentifierLinkedMap, UkeyIndexSet};
use rspack_core::{
  Chunk, ChunkGraph, ChunkGroupByUkey, ChunkGroupUkey, ChunkUkey, Compilation, PathData,
  RuntimeGlobals, SourceType, get_js_chunk_filename_template,
  rspack_sources::{BoxSource, RawStringSource, SourceExt},
};
use rspack_error::{Result, error};
use rspack_hash::RspackHash;
use rspack_plugin_javascript::runtime::stringify_chunks_to_array;
use rustc_hash::FxHashSet as HashSet;

pub fn update_hash_for_entry_startup(
  hasher: &mut RspackHash,
  compilation: &Compilation,
  entries: &IdentifierLinkedMap<ChunkGroupUkey>,
  chunk: &ChunkUkey,
) {
  for (module, entry) in entries {
    if let Some(module_id) = compilation
      .get_module_graph()
      .module_graph_module_by_identifier(module)
      .and_then(|module| {
        ChunkGraph::get_module_id(&compilation.module_ids_artifact, module.module_identifier)
      })
    {
      module_id.hash(hasher);
    }

    if let Some(runtime_chunk) = compilation
      .chunk_group_by_ukey
      .get(entry)
      .map(|e| e.get_runtime_chunk(&compilation.chunk_group_by_ukey))
    {
      for chunk_ukey in get_all_chunks(
        entry,
        chunk,
        Some(&runtime_chunk),
        &compilation.chunk_group_by_ukey,
      ) {
        if let Some(chunk) = compilation.chunk_by_ukey.get(&chunk_ukey) {
          chunk.id(&compilation.chunk_ids_artifact).hash(hasher);
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
) -> UkeyIndexSet<ChunkUkey> {
  fn add_chunks(
    chunk_group_by_ukey: &ChunkGroupByUkey,
    chunks: &mut UkeyIndexSet<ChunkUkey>,
    entrypoint_ukey: &ChunkGroupUkey,
    exclude_chunk1: &ChunkUkey,
    exclude_chunk2: Option<&ChunkUkey>,
    visit_chunk_groups: &mut UkeyIndexSet<ChunkGroupUkey>,
  ) {
    if let Some(entrypoint) = chunk_group_by_ukey.get(entrypoint_ukey) {
      for chunk in &entrypoint.chunks {
        if chunk == exclude_chunk1 {
          continue;
        }
        if let Some(exclude_chunk2) = exclude_chunk2
          && chunk == exclude_chunk2
        {
          continue;
        }
        chunks.insert(*chunk);
      }

      for parent in entrypoint.parents_iterable() {
        if visit_chunk_groups.contains(parent) {
          continue;
        }
        visit_chunk_groups.insert(*parent);
        if let Some(chunk_group) = chunk_group_by_ukey.get(parent)
          && chunk_group.is_initial()
        {
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

  let mut chunks = UkeyIndexSet::default();
  let mut visit_chunk_groups = UkeyIndexSet::default();

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

pub async fn get_runtime_chunk_output_name(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
) -> Result<String> {
  let entry_point = {
    let entry_points = compilation
      .chunk_graph
      .get_chunk_entry_modules_with_chunk_group_iterable(chunk_ukey);

    let (_, entry_point_ukey) = entry_points
      .iter()
      .next()
      .ok_or_else(|| error!("should has entry point ukey"))?;

    compilation.chunk_group_by_ukey.expect_get(entry_point_ukey)
  };

  let runtime_chunk = compilation
    .chunk_by_ukey
    .expect_get(&entry_point.get_runtime_chunk(&compilation.chunk_group_by_ukey));

  get_chunk_output_name(runtime_chunk, compilation).await
}

pub async fn runtime_chunk_has_hash(
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
) -> Result<bool> {
  let entry_point = {
    let entry_points = compilation
      .chunk_graph
      .get_chunk_entry_modules_with_chunk_group_iterable(chunk_ukey);

    let (_, entry_point_ukey) = entry_points
      .iter()
      .next()
      .ok_or_else(|| error!("should has entry point ukey"))?;

    compilation.chunk_group_by_ukey.expect_get(entry_point_ukey)
  };

  let runtime_chunk_ukey = entry_point.get_runtime_chunk(&compilation.chunk_group_by_ukey);
  let runtime_chunk = compilation.chunk_by_ukey.expect_get(&runtime_chunk_ukey);

  let filename = get_js_chunk_filename_template(
    runtime_chunk,
    &compilation.options.output,
    &compilation.chunk_group_by_ukey,
  );

  if filename.has_hash_placeholder() {
    return Ok(true);
  }

  if filename.has_content_hash_placeholder()
    && (compilation
      .chunk_graph
      .has_chunk_full_hash_modules(&runtime_chunk_ukey, &compilation.runtime_modules)
      || compilation
        .chunk_graph
        .has_chunk_dependent_hash_modules(&runtime_chunk_ukey, &compilation.runtime_modules))
  {
    return Ok(true);
  }

  Ok(false)
}

pub fn generate_entry_startup(
  compilation: &Compilation,
  chunk: &ChunkUkey,
  entries: &IdentifierLinkedMap<ChunkGroupUkey>,
  passive: bool,
) -> BoxSource {
  let mut module_id_exprs = vec![];
  let mut chunks_ids = HashSet::default();
  let module_graph = compilation.get_module_graph();
  for (module, entry) in entries {
    if let Some(module_id) = module_graph
      .module_by_identifier(module)
      .filter(|module| {
        module
          .source_types(&module_graph)
          .contains(&SourceType::JavaScript)
      })
      .and_then(|module| {
        ChunkGraph::get_module_id(&compilation.module_ids_artifact, module.identifier())
      })
    {
      let module_id_expr = serde_json::to_string(module_id).expect("invalid module_id");
      module_id_exprs.push(module_id_expr);
    } else {
      continue;
    }

    if let Some(runtime_chunk) = compilation
      .chunk_group_by_ukey
      .get(entry)
      .map(|e| e.get_runtime_chunk(&compilation.chunk_group_by_ukey))
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
            let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
            chunk.expect_id(&compilation.chunk_ids_artifact).clone()
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

  let module_ids_code = &module_id_exprs
    .iter()
    .map(|module_id_expr| format!("__webpack_exec__({module_id_expr})"))
    .collect::<Vec<_>>()
    .join(", ");
  if chunks_ids.is_empty() {
    if !module_ids_code.is_empty() {
      source.push_str("var __webpack_exports__ = (");
      source.push_str(module_ids_code);
      source.push_str(");\n");
    }
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

  RawStringSource::from(source).boxed()
}

/**
 * This is ported from https://github.com/webpack/webpack/blob/87660921808566ef3b8796f8df61bd79fc026108/lib/esm/ModuleChunkFormatPlugin.js#L98
 */
pub fn get_relative_path(base_chunk_output_name: &str, other_chunk_output_name: &str) -> String {
  let mut base_chunk_output_name_arr = base_chunk_output_name.split('/').collect::<Vec<_>>();
  base_chunk_output_name_arr.pop();
  let mut other_chunk_output_name_arr = other_chunk_output_name.split('/').collect::<Vec<_>>();
  while !base_chunk_output_name_arr.is_empty()
    && !other_chunk_output_name_arr.is_empty()
    && base_chunk_output_name_arr[0] == other_chunk_output_name_arr[0]
  {
    base_chunk_output_name_arr.remove(0);
    other_chunk_output_name_arr.remove(0);
  }
  let path = if base_chunk_output_name_arr.is_empty() {
    "./".to_string()
  } else {
    "../".repeat(base_chunk_output_name_arr.len())
  };
  format!("{path}{}", other_chunk_output_name_arr.join("/"))
}

pub async fn get_chunk_output_name(chunk: &Chunk, compilation: &Compilation) -> Result<String> {
  let hash = chunk.rendered_hash(
    &compilation.chunk_hashes_artifact,
    compilation.options.output.hash_digest_length,
  );
  let filename = get_js_chunk_filename_template(
    chunk,
    &compilation.options.output,
    &compilation.chunk_group_by_ukey,
  );
  compilation
    .get_path(
      &filename,
      PathData::default()
        .chunk_id_optional(
          chunk
            .id(&compilation.chunk_ids_artifact)
            .map(|id| id.as_str()),
        )
        .chunk_hash_optional(chunk.rendered_hash(
          &compilation.chunk_hashes_artifact,
          compilation.options.output.hash_digest_length,
        ))
        .chunk_name_optional(chunk.name_for_filename_template(&compilation.chunk_ids_artifact))
        .content_hash_optional(chunk.rendered_content_hash_by_source_type(
          &compilation.chunk_hashes_artifact,
          &SourceType::JavaScript,
          compilation.options.output.hash_digest_length,
        ))
        .hash_optional(hash),
    )
    .await
}

pub fn get_chunk_runtime_requirements<'a>(
  compilation: &'a Compilation,
  chunk_ukey: &ChunkUkey,
) -> &'a RuntimeGlobals {
  ChunkGraph::get_chunk_runtime_requirements(compilation, chunk_ukey)
}
