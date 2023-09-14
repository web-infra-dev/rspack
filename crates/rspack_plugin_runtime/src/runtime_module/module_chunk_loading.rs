use rspack_core::{
  rspack_sources::{BoxSource, ConcatSource, RawSource, SourceExt},
  Chunk, ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleStage,
};
use rspack_identifier::Identifier;

use super::utils::{chunk_has_js, get_output_dir};
use crate::impl_runtime_module;
use crate::runtime_module::utils::{get_initial_chunk_ids, stringify_chunks};

#[derive(Debug, Default, Eq)]
pub struct ModuleChunkLoadingRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
  runtime_requirements: RuntimeGlobals,
}

impl ModuleChunkLoadingRuntimeModule {
  pub fn new(runtime_requirements: RuntimeGlobals) -> Self {
    Self {
      id: Identifier::from("webpack/runtime/module_chunk_loading"),
      chunk: None,
      runtime_requirements,
    }
  }

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

  fn generate(&self, compilation: &Compilation) -> BoxSource {
    let chunk = compilation
      .chunk_by_ukey
      .get(&self.chunk.expect("The chunk should be attached."))
      .expect("Chunk is not found, make sure you had attach chunkUkey successfully.");
    let with_hmr = self
      .runtime_requirements
      .contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS);
    let with_external_install_chunk = self
      .runtime_requirements
      .contains(RuntimeGlobals::EXTERNAL_INSTALL_CHUNK);
    let mut source = ConcatSource::default();
    let root_output_dir = get_output_dir(chunk, compilation, true);
    if self.runtime_requirements.contains(RuntimeGlobals::BASE_URI) {
      source.add(self.generate_base_uri(chunk, compilation, &root_output_dir));
    }

    // object to store loaded and loading chunks
    // undefined = chunk not loaded, null = chunk preloaded/prefetched
    // [resolve, reject, Promise] = chunk loading, 0 = chunk loaded
    let initial_chunks = get_initial_chunk_ids(self.chunk, compilation, chunk_has_js);
    if with_hmr {
      let state_expression = format!("{}_module", RuntimeGlobals::HMR_RUNTIME_STATE_PREFIX);
      source.add(RawSource::from(format!(
        "var installedChunks = {} = {} || {};\n",
        state_expression,
        state_expression,
        &stringify_chunks(&initial_chunks, 1)
      )));
    } else {
      source.add(RawSource::from(format!(
        "var installedChunks = {};\n",
        &stringify_chunks(&initial_chunks, 0)
      )));
    }

    let with_loading = self
      .runtime_requirements
      .contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS);
    let with_on_chunk_load = self
      .runtime_requirements
      .contains(RuntimeGlobals::ON_CHUNKS_LOADED);

    if with_loading || with_external_install_chunk {
      source.add(RawSource::from(
        include_str!("runtime/module_chunk_loading.js").replace(
          "$withOnChunkLoad$",
          match with_on_chunk_load {
            true => "__webpack_require__.O();",
            false => "",
          },
        ),
      ));
    }

    if with_loading {
      source.add(RawSource::from(
        include_str!("runtime/module_chunk_loading_with_loading.js")
          // TODO
          .replace("JS_MATCHER", "chunkId")
          .replace(
            "$importFunctionName$",
            &compilation.options.output.import_function_name,
          )
          .replace("$OUTPUT_DIR$", &root_output_dir),
      ));
    }

    if with_external_install_chunk {
      source.add(RawSource::from("__webpack_require__.C = installChunk;\n"));
    }

    if with_on_chunk_load {
      source.add(RawSource::from(format!(
        r#"{}.j = function(chunkId) {{
            return installedChunks[chunkId] === 0;
        }}"#,
        RuntimeGlobals::ON_CHUNKS_LOADED
      )));
    }

    source.boxed()
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }
}

impl_runtime_module!(ModuleChunkLoadingRuntimeModule);
