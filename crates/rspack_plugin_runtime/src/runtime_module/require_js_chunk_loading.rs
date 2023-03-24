use rspack_core::{
  get_js_chunk_filename_template,
  rspack_sources::{BoxSource, ConcatSource, RawSource, SourceExt},
  ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule, SourceType, RUNTIME_MODULE_STAGE_ATTACH,
};
use rspack_identifier::Identifier;

use super::utils::{chunk_has_js, get_undo_path};
use crate::impl_runtime_module;
use crate::runtime_module::utils::{get_initial_chunk_ids, stringify_chunks};

#[derive(Debug, Default, Eq)]
pub struct RequireChunkLoadingRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
  runtime_requirements: RuntimeGlobals,
}

impl RequireChunkLoadingRuntimeModule {
  pub fn new(runtime_requirements: RuntimeGlobals) -> Self {
    Self {
      id: Identifier::from("webpack/runtime/require_chunk_loading"),
      chunk: None,
      runtime_requirements,
    }
  }
}

impl RuntimeModule for RequireChunkLoadingRuntimeModule {
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
    let initial_chunks = get_initial_chunk_ids(self.chunk, compilation, chunk_has_js);
    let mut source = ConcatSource::default();

    if with_hmr {
      source.add(RawSource::from(format!(
        "var installedChunks = {} = {} || {};\n",
        RuntimeGlobals::HMR_RUNTIME_STATE_PREFIX,
        RuntimeGlobals::HMR_RUNTIME_STATE_PREFIX,
        &stringify_chunks(&initial_chunks, 0)
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

    if with_loading || with_external_install_chunk {
      source.add(RawSource::from(include_str!(
        "runtime/require_chunk_loading.js"
      )));
    }

    if with_loading {
      let filename = get_js_chunk_filename_template(
        chunk,
        &compilation.options.output,
        &compilation.chunk_group_by_ukey,
      );
      let output_dir = filename.render_with_chunk(chunk, ".js", &SourceType::JavaScript);
      source.add(RawSource::from(
        include_str!("runtime/require_chunk_loading_with_loading.js")
          // TODO
          .replace("JS_MATCHER", "chunkId")
          .replace(
            "$OUTPUT_DIR$",
            &get_undo_path(
              output_dir.as_str(),
              compilation.options.output.path.display().to_string(),
              true,
            ),
          ),
      ));
    }

    if with_hmr {
      source.add(RawSource::from(include_str!(
        "runtime/require_chunk_loading_with_hmr.js"
      )));
      source.add(RawSource::from(
        include_str!("runtime/javascript_hot_module_replacement.js").replace("$key$", "jsonp"),
      ));
    }

    if self
      .runtime_requirements
      .contains(RuntimeGlobals::HMR_DOWNLOAD_MANIFEST)
    {
      source.add(RawSource::from(include_str!(
        "runtime/require_chunk_loading_with_hmr_manifest.js"
      )));
    }

    if self
      .runtime_requirements
      .contains(RuntimeGlobals::ON_CHUNKS_LOADED)
    {
      source.add(RawSource::from(include_str!(
        "runtime/require_chunk_loading_with_on_chunk_load.js"
      )));
    }

    if with_external_install_chunk {
      source.add(RawSource::from(include_str!(
        "runtime/require_chunk_loading_with_external_install_chunk.js"
      )));
    }

    source.boxed()
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }

  fn stage(&self) -> u8 {
    RUNTIME_MODULE_STAGE_ATTACH
  }
}

impl_runtime_module!(RequireChunkLoadingRuntimeModule);
