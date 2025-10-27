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

  if chunk_needs_mf_async_startup(compilation, chunk)
    && let Some(chunk_ref) = compilation.chunk_by_ukey.get(chunk)
  {
    chunk_ref.id(&compilation.chunk_ids_artifact).hash(hasher);
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

pub fn chunk_contains_container_entry(compilation: &Compilation, chunk: &ChunkUkey) -> bool {
  let module_graph = compilation.get_module_graph();
  let has_container_entry = compilation
    .chunk_graph
    .get_chunk_entry_modules_with_chunk_group_iterable(chunk)
    .keys()
    .any(|identifier| {
      module_graph
        .module_by_identifier(identifier)
        .map(|module| module.identifier().as_str().starts_with("container entry"))
        .unwrap_or(false)
    });
  #[cfg(debug_assertions)]
  {
    if has_container_entry
      && let Some(chunk_ref) = compilation.chunk_by_ukey.get(chunk)
      && let Some(chunk_id) = chunk_ref.id(&compilation.chunk_ids_artifact)
    {
      let ids: Vec<_> = compilation
        .chunk_graph
        .get_chunk_entry_modules_with_chunk_group_iterable(chunk)
        .keys()
        .filter_map(|identifier| {
          module_graph
            .module_by_identifier(identifier)
            .map(|module| module.identifier().as_str().to_string())
        })
        .collect();
      eprintln!(
        "[mf-debug] chunk {} detected container entries: {:?}",
        chunk_id, ids
      );
    }
  }
  has_container_entry
}

pub fn chunk_needs_mf_async_startup(compilation: &Compilation, chunk: &ChunkUkey) -> bool {
  let Some(chunk_ref) = compilation.chunk_by_ukey.get(chunk) else {
    return false;
  };

  if !compilation.options.experiments.mf_async_startup {
    return false;
  }

  let Some(runtime_requirements) = compilation.cgc_runtime_requirements_artifact.get(chunk) else {
    return false;
  };

  // Check for MF-specific runtime requirements (not just ENSURE_CHUNK_HANDLERS which is too broad)
  let has_mf_runtime = runtime_requirements.contains(RuntimeGlobals::INITIALIZE_SHARING)
    || runtime_requirements.contains(RuntimeGlobals::SHARE_SCOPE_MAP)
    || runtime_requirements.contains(RuntimeGlobals::CURRENT_REMOTE_GET_SCOPE);

  if let Some(chunk_id) = chunk_ref.id(&compilation.chunk_ids_artifact)
    && chunk_id.as_str() == "build time chunk"
  {
    return false;
  }

  if compilation.chunk_graph.get_number_of_entry_modules(chunk) == 0 {
    return false;
  }

  // Check for MF-specific module types
  let module_graph = compilation.get_module_graph();
  let has_remote_modules = !compilation
    .chunk_graph
    .get_chunk_modules_by_source_type(chunk, SourceType::Remote, &module_graph)
    .is_empty();
  let has_consume_modules = !compilation
    .chunk_graph
    .get_chunk_modules_by_source_type(chunk, SourceType::ConsumeShared, &module_graph)
    .is_empty();

  // Only enable async startup if chunk actually participates in MF
  if !has_mf_runtime && !has_remote_modules && !has_consume_modules {
    return false;
  }

  #[cfg(debug_assertions)]
  {
    if let Some(chunk_id) = chunk_ref.id(&compilation.chunk_ids_artifact)
      && chunk_id.as_str().contains("container")
    {
      let entries: Vec<_> = compilation
        .chunk_graph
        .get_chunk_entry_modules_with_chunk_group_iterable(chunk)
        .keys()
        .filter_map(|identifier| {
          module_graph
            .module_by_identifier(identifier)
            .map(|module| module.identifier().as_str().to_string())
        })
        .collect();
      eprintln!("[mf-debug] chunk {} entry modules {:?}", chunk_id, entries);
    }
  }

  // Check if this chunk contains a container entry module
  // Container entries should NOT have async startup - only host applications should
  let has_container_entry = chunk_contains_container_entry(compilation, chunk);
  let has_expose_modules = !compilation
    .chunk_graph
    .get_chunk_modules_identifier_by_source_type(chunk, SourceType::Expose, &module_graph)
    .is_empty();

  // Return false for container entries (they should not have async startup)
  if has_container_entry || has_expose_modules {
    #[cfg(debug_assertions)]
    {
      if let Some(chunk_id) = chunk_ref.id(&compilation.chunk_ids_artifact) {
        eprintln!(
          "[mf-debug] disabling async startup for chunk {} (container_entry={}, expose_modules={})",
          chunk_id, has_container_entry, has_expose_modules
        );
      }
    }
    return false;
  }

  #[cfg(debug_assertions)]
  {
    if let Some(chunk_id) = chunk_ref.id(&compilation.chunk_ids_artifact) {
      eprintln!(
        "[mf-debug] enabling async startup for chunk {} (has_mf_runtime={}, has_remote_modules={}, has_consume_modules={})",
        chunk_id, has_mf_runtime, has_remote_modules, has_consume_modules
      );
    }
  }

  true
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
  let mf_async_startup = chunk_needs_mf_async_startup(compilation, chunk);

  if chunks_ids.is_empty() && !mf_async_startup {
    if !module_ids_code.is_empty() {
      source.push_str("var __webpack_exports__ = (");
      source.push_str(module_ids_code);
      source.push_str(");\n");
    }
  } else {
    let chunk_ids_literal = stringify_chunks_to_array(&chunks_ids);
    if mf_async_startup {
      let startup_global = RuntimeGlobals::STARTUP.name();
      let ensure_chunk_handlers = RuntimeGlobals::ENSURE_CHUNK_HANDLERS.name();

      source.push_str(&format!("var chunkIds = {};\n", chunk_ids_literal));
      source.push_str("var promises = [];\n");
      source.push_str(&format!(
        "if (typeof {startup} === \"function\") {{\n  {startup}();\n}} else {{\n  console.warn(\"[Module Federation] {startup} is not a function, skipping startup extension\");\n}}\n",
        startup = startup_global
      ));
      source.push_str(
        "var __federation__ = __webpack_require__.federation;\nif (__federation__ && typeof __federation__.installRuntime === \"function\") {\n  __federation__.installRuntime();\n}\n",
      );
      source.push_str(&format!(
        "var __chunk_handlers__ = {};\n",
        ensure_chunk_handlers
      ));
      source.push_str(
        "var __handler_chunk_ids__ = chunkIds.slice();\nif (__webpack_require__.remotesLoadingData && __webpack_require__.remotesLoadingData.chunkMapping) {\n  for (var __chunk_key__ in __webpack_require__.remotesLoadingData.chunkMapping) {\n    if (__handler_chunk_ids__.indexOf(__chunk_key__) < 0) __handler_chunk_ids__.push(__chunk_key__);\n  }\n}\nif (__webpack_require__.consumesLoadingData && __webpack_require__.consumesLoadingData.chunkMapping) {\n  for (var __consume_chunk__ in __webpack_require__.consumesLoadingData.chunkMapping) {\n    if (__handler_chunk_ids__.indexOf(__consume_chunk__) < 0) __handler_chunk_ids__.push(__consume_chunk__);\n  }\n}\n"
      );
      source.push_str("if (__chunk_handlers__) {\n");
      source.push_str("  var __handler_list__ = [\n");
      source.push_str("    __chunk_handlers__.consumes || function(chunkId, promises) {},\n");
      source.push_str("    __chunk_handlers__.remotes || function(chunkId, promises) {}\n");
      source.push_str("  ];\n");
      source.push_str(
        "  promises = __handler_list__.reduce(function(p, handler) {\n    if (typeof handler === \"function\") {\n      for (var idx = 0; idx < __handler_chunk_ids__.length; idx++) {\n        handler(__handler_chunk_ids__[idx], p);\n      }\n    }\n    return p;\n  }, promises);\n"
      );
      source.push_str("}\n");
      source.push_str("var __webpack_exports__ = Promise.all(promises).then(function() {\n");
      if passive {
        source.push_str(&format!(
          "  {on_chunks}(0, chunkIds, function() {{\n    return {modules};\n  }});\n  return {on_chunks}();\n",
          on_chunks = RuntimeGlobals::ON_CHUNKS_LOADED.name(),
          modules = module_ids_code
        ));
      } else {
        source.push_str(&format!(
          "  return {startup_entry}(0, chunkIds, function() {{\n    return {modules};\n  }});\n",
          startup_entry = RuntimeGlobals::STARTUP_ENTRYPOINT.name(),
          modules = module_ids_code
        ));
      }
      source.push_str("});\n");
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
        chunk_ids_literal,
        module_ids_code
      ));
      if passive {
        source.push_str(&format!(
          "var __webpack_exports__ = {}();\n",
          RuntimeGlobals::ON_CHUNKS_LOADED
        ));
      }
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
