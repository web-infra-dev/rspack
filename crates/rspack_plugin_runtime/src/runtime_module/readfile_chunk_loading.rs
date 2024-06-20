use rspack_core::{
  compile_boolean_matcher, impl_runtime_module,
  rspack_sources::{BoxSource, ConcatSource, RawSource, SourceExt},
  BooleanMatcher, Chunk, ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleStage,
};
use rspack_identifier::Identifier;

use super::{
  generate_javascript_hmr_runtime,
  utils::{chunk_has_js, get_output_dir},
};
use crate::{
  get_chunk_runtime_requirements,
  runtime_module::utils::{get_initial_chunk_ids, stringify_chunks},
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct ReadFileChunkLoadingRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl Default for ReadFileChunkLoadingRuntimeModule {
  fn default() -> Self {
    Self::with_default(
      Identifier::from("webpack/runtime/readfile_chunk_loading"),
      None,
    )
  }
}

impl ReadFileChunkLoadingRuntimeModule {
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
          "require(\"url\").pathToFileURL({})",
          if !root_output_dir.is_empty() {
            format!(
              "__dirname + {}",
              serde_json::to_string(&format!("/{root_output_dir}"))
                .expect("should able to be serde_json::to_string")
            )
          } else {
            "__filename".to_string()
          }
        )
      });
    RawSource::from(format!("{} = {};\n", RuntimeGlobals::BASE_URI, base_uri)).boxed()
  }
}

impl RuntimeModule for ReadFileChunkLoadingRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }
  fn generate(&self, compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    let chunk = compilation
      .chunk_by_ukey
      .expect_get(&self.chunk.expect("The chunk should be attached."));
    let runtime_requirements = get_chunk_runtime_requirements(compilation, &chunk.ukey);

    let with_base_uri = runtime_requirements.contains(RuntimeGlobals::BASE_URI);
    let with_hmr = runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS);
    let with_hmr_manifest = runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_MANIFEST);
    let with_external_install_chunk =
      runtime_requirements.contains(RuntimeGlobals::EXTERNAL_INSTALL_CHUNK);
    let with_loading = runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS);
    let with_on_chunk_load = runtime_requirements.contains(RuntimeGlobals::ON_CHUNKS_LOADED);

    let condition_map =
      compilation
        .chunk_graph
        .get_chunk_condition_map(&chunk.ukey, compilation, chunk_has_js);
    let has_js_matcher = compile_boolean_matcher(&condition_map);

    let initial_chunks = get_initial_chunk_ids(self.chunk, compilation, chunk_has_js);
    let root_output_dir = get_output_dir(chunk, compilation, false)?;
    let mut source = ConcatSource::default();

    if with_base_uri {
      source.add(self.generate_base_uri(chunk, compilation, &root_output_dir));
    }

    if with_hmr {
      let state_expression = format!("{}_readFileVm", RuntimeGlobals::HMR_RUNTIME_STATE_PREFIX);
      source.add(RawSource::from(format!(
        "var installedChunks = {} = {} || {};\n",
        state_expression,
        state_expression,
        &stringify_chunks(&initial_chunks, 0)
      )));
    } else {
      source.add(RawSource::from(format!(
        "var installedChunks = {};\n",
        &stringify_chunks(&initial_chunks, 0)
      )));
    }

    if with_on_chunk_load {
      source.add(RawSource::from(include_str!(
        "runtime/readfile_chunk_loading_with_on_chunk_load.js"
      )));
    }

    if with_loading || with_external_install_chunk {
      source.add(RawSource::from(
        include_str!("runtime/readfile_chunk_loading.js").replace(
          "$WITH_ON_CHUNK_LOADED$",
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
        include_str!("runtime/readfile_chunk_loading_with_loading.js")
          .replace("$JS_MATCHER$", &has_js_matcher.render("chunkId"))
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
        // ReadFile + VM.run chunk loading for javascript"
        __webpack_require__.f.readFileVm = function (chunkId, promises) {{
          {body}
        }};
        "#
      )));
    }

    if with_external_install_chunk {
      source.add(RawSource::from(include_str!(
        "runtime/readfile_chunk_loading_with_external_install_chunk.js"
      )));
    }

    if with_hmr {
      source.add(RawSource::from(include_str!(
        "runtime/readfile_chunk_loading_with_hmr.js"
      )));
      source.add(RawSource::from(generate_javascript_hmr_runtime(
        "readFileVm",
      )));
    }

    if with_hmr_manifest {
      source.add(RawSource::from(
        include_str!("runtime/readfile_chunk_loading_with_hmr_manifest.js")
          .replace("$OUTPUT_DIR$", &root_output_dir),
      ));
    }

    Ok(source.boxed())
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk)
  }
  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }
}
