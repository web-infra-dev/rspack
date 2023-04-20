use rspack_core::{
  rspack_sources::{BoxSource, ConcatSource, RawSource, SourceExt},
  ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule, RUNTIME_MODULE_STAGE_ATTACH,
};
use rspack_identifier::Identifier;

use super::utils::{chunk_has_js, get_output_dir};
use crate::impl_runtime_module;
use crate::runtime_module::utils::{get_initial_chunk_ids, stringify_chunks};

#[derive(Debug, Default, Eq)]
pub struct ImportScriptsChunkLoadingRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
  runtime_requirements: RuntimeGlobals,
  with_create_script_url: bool,
}

impl ImportScriptsChunkLoadingRuntimeModule {
  pub fn new(runtime_requirements: RuntimeGlobals, with_create_script_url: bool) -> Self {
    Self {
      id: Identifier::from("webpack/runtime/import_scripts_chunk_loading"),
      chunk: None,
      runtime_requirements,
      with_create_script_url,
    }
  }
}

impl RuntimeModule for ImportScriptsChunkLoadingRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, compilation: &Compilation) -> BoxSource {
    let chunk = compilation
      .chunk_by_ukey
      .get(&self.chunk.expect("The chunk should be attached."))
      .expect("Chunk is not found, make sure you had attach chunkUkey successfully.");
    let initial_chunks = get_initial_chunk_ids(self.chunk, compilation, chunk_has_js);
    let with_hmr = self
      .runtime_requirements
      .contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS);
    let mut source = ConcatSource::default();
    let root_output_dir = get_output_dir(chunk, compilation, false);

    if self.runtime_requirements.contains(RuntimeGlobals::BASE_URI) {
      // TODO EntryOptions.baseURI
      source.add(RawSource::from(format!(
        "{} = self.location + {};\n",
        RuntimeGlobals::BASE_URI,
        if root_output_dir.is_empty() {
          "''".to_string()
        } else {
          format!("/../{root_output_dir}")
        }
      )))
    }

    // object to store loaded chunks
    // "1" means "already loaded"
    if with_hmr {
      let state_expression = format!("{}_importScripts", RuntimeGlobals::HMR_RUNTIME_STATE_PREFIX);
      source.add(RawSource::from(format!(
        "var installedChunks = {} = {} || {};\n",
        state_expression,
        state_expression,
        &stringify_chunks(&initial_chunks, 1)
      )));
    } else {
      source.add(RawSource::from(format!(
        "var installedChunks = {};\n",
        &stringify_chunks(&initial_chunks, 1)
      )));
    }

    let with_loading = self
      .runtime_requirements
      .contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS);

    if with_loading {
      let chunk_loading_global_expr = format!(
        "{}['{}']",
        &compilation.options.output.global_object, &compilation.options.output.chunk_loading_global
      );
      let url = if self.with_create_script_url {
        format!(
          "{}({} + {}(chunkId))",
          RuntimeGlobals::CREATE_SCRIPT_URL,
          RuntimeGlobals::PUBLIC_PATH,
          RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME
        )
      } else {
        format!(
          "{} + {}(chunkId)",
          RuntimeGlobals::PUBLIC_PATH,
          RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME
        )
      };
      source.add(RawSource::from(
        include_str!("runtime/import_scripts_chunk_loading.js")
          // TODO
          .replace("JS_MATCHER", "chunkId")
          .replace("$URL$", &url)
          .replace("$CHUNK_LOADING_GLOBAL_EXPR$", &chunk_loading_global_expr),
      ));
    }

    if with_hmr {
      let url = if self.with_create_script_url {
        format!(
          "{}({} + {}(chunkId))",
          RuntimeGlobals::CREATE_SCRIPT_URL,
          RuntimeGlobals::PUBLIC_PATH,
          RuntimeGlobals::GET_CHUNK_UPDATE_SCRIPT_FILENAME
        )
      } else {
        format!(
          "{} + {}(chunkId)",
          RuntimeGlobals::PUBLIC_PATH,
          RuntimeGlobals::GET_CHUNK_UPDATE_SCRIPT_FILENAME
        )
      };
      source.add(RawSource::from(
        include_str!("runtime/import_scripts_chunk_loading_with_hmr.js")
          .replace("$URL$", &url)
          .replace("$globalObject$", &compilation.options.output.global_object),
      ));
      source.add(RawSource::from(
        include_str!("runtime/javascript_hot_module_replacement.js")
          .replace("$key$", "importScrips"),
      ));
    }

    if self
      .runtime_requirements
      .contains(RuntimeGlobals::HMR_DOWNLOAD_MANIFEST)
    {
      // TODO: import_scripts_chunk_loading_with_hmr_manifest same as jsonp_chunk_loading_with_hmr_manifest
      source.add(RawSource::from(include_str!(
        "runtime/import_scripts_chunk_loading_with_hmr_manifest.js"
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

impl_runtime_module!(ImportScriptsChunkLoadingRuntimeModule);
