use rspack_core::{
  compile_boolean_matcher, impl_runtime_module,
  rspack_sources::{BoxSource, ConcatSource, RawSource, SourceExt},
  BooleanMatcher, Chunk, ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleStage,
};
use rspack_identifier::Identifier;
use rspack_util::source_map::SourceMapKind;

use super::utils::{chunk_has_js, get_output_dir};
use crate::{
  get_chunk_runtime_requirements,
  runtime_module::utils::{get_initial_chunk_ids, stringify_chunks},
};

#[impl_runtime_module]
#[derive(Debug, Eq)]
pub struct ModuleChunkLoadingRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl Default for ModuleChunkLoadingRuntimeModule {
  fn default() -> Self {
    Self {
      id: Identifier::from("webpack/runtime/module_chunk_loading"),
      chunk: None,
      source_map_kind: SourceMapKind::empty(),
      custom_source: None,
    }
  }
}

impl ModuleChunkLoadingRuntimeModule {
  fn generate_base_uri(
    &self,
    chunk: &Chunk,
    compilation: &Compilation,
    root_output_dir: &str,
  ) -> BoxSource {
    let base_uri = chunk
      .get_entry_options(&compilation.chunk_group_by_ukey)
      .and_then(|options| options.base_uri.as_ref())
      .and_then(|base_uri| serde_json::to_string(base_uri).ok())
      .unwrap_or_else(|| {
        format!(
          "new URL({}, import.meta.url);",
          serde_json::to_string(root_output_dir).expect("should able to be serde_json::to_string")
        )
      });
    RawSource::from(format!("{} = {};\n", RuntimeGlobals::BASE_URI, base_uri)).boxed()
  }
}

impl RuntimeModule for ModuleChunkLoadingRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    let chunk = compilation
      .chunk_by_ukey
      .expect_get(&self.chunk.expect("The chunk should be attached."));
    let runtime_requirements = get_chunk_runtime_requirements(compilation, &chunk.ukey);

    let with_base_uri = runtime_requirements.contains(RuntimeGlobals::BASE_URI);
    let with_external_install_chunk =
      runtime_requirements.contains(RuntimeGlobals::EXTERNAL_INSTALL_CHUNK);
    let with_loading = runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS);
    let with_on_chunk_load = runtime_requirements.contains(RuntimeGlobals::ON_CHUNKS_LOADED);
    let with_hmr = runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS);

    let condition_map =
      compilation
        .chunk_graph
        .get_chunk_condition_map(&chunk.ukey, compilation, chunk_has_js);
    let has_js_matcher = compile_boolean_matcher(&condition_map);
    let initial_chunks = get_initial_chunk_ids(self.chunk, compilation, chunk_has_js);

    let root_output_dir = get_output_dir(chunk, compilation, true)?;

    let mut source = ConcatSource::default();

    if with_base_uri {
      source.add(self.generate_base_uri(chunk, compilation, &root_output_dir));
    }

    source.add(RawSource::from(format!(
      r#"
      // object to store loaded and loading chunks
      // undefined = chunk not loaded, null = chunk preloaded/prefetched
      // [resolve, reject, Promise] = chunk loading, 0 = chunk loaded
      var installedChunks = {}{};
      "#,
      match with_hmr {
        true => {
          let state_expression = format!("{}_module", RuntimeGlobals::HMR_RUNTIME_STATE_PREFIX);
          format!("{} = {} || ", state_expression, state_expression)
        }
        false => "".to_string(),
      },
      &stringify_chunks(&initial_chunks, 0)
    )));

    if with_loading || with_external_install_chunk {
      source.add(RawSource::from(
        include_str!("runtime/module_chunk_loading.js").replace(
          "$WITH_ON_CHUNK_LOAD$",
          match with_on_chunk_load {
            true => "__webpack_require__.O();",
            false => "",
          },
        ),
      ));
    }

    if with_loading {
      let body = if matches!(has_js_matcher, BooleanMatcher::Condition(false)) {
        "installedChunks[chunkId] = 0;".to_string()
      } else {
        include_str!("runtime/module_chunk_loading_with_loading.js")
          .replace("$JS_MATCHER$", &has_js_matcher.render("chunkId"))
          .replace(
            "$IMPORT_FUNCTION_NAME$",
            &compilation.options.output.import_function_name,
          )
          .replace("$OUTPUT_DIR$", &root_output_dir)
          .replace(
            "$MATCH_FALLBACK$",
            if matches!(has_js_matcher, BooleanMatcher::Condition(true)) {
              ""
            } else {
              "else installedChunks[chunkId] = 0;\n"
            },
          )
      };

      source.add(RawSource::from(format!(
        r#"
        {}.j = function (chunkId, promises) {{
          {body}
        }}
        "#,
        RuntimeGlobals::ENSURE_CHUNK_HANDLERS
      )));
    }

    if with_external_install_chunk {
      source.add(RawSource::from(format!(
        r#"
        {} = installChunk;
        "#,
        RuntimeGlobals::EXTERNAL_INSTALL_CHUNK
      )));
    }

    if with_on_chunk_load {
      source.add(RawSource::from(format!(
        r#"
        {}.j = function(chunkId) {{
            return installedChunks[chunkId] === 0;
        }}
        "#,
        RuntimeGlobals::ON_CHUNKS_LOADED
      )));
    }

    Ok(source.boxed())
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }
}
