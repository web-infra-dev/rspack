use rspack_core::{
  rspack_sources::{BoxSource, ConcatSource, RawSource, SourceExt},
  runtime_globals, ChunkUkey, Compilation, RuntimeModule, RUNTIME_MODULE_STAGE_ATTACH,
};
use rustc_hash::FxHashSet as HashSet;
use serde_json::json;

use super::utils::chunk_has_js;
use crate::runtime_module::utils::{get_initial_chunk_ids, stringify_chunks};

#[derive(Debug, Default)]
pub struct ReadFileChunkLoadingRuntimeModule {
  chunk: Option<ChunkUkey>,
  runtime_requirements: HashSet<&'static str>,
}

impl ReadFileChunkLoadingRuntimeModule {
  pub fn new(runtime_requirements: HashSet<&'static str>) -> Self {
    Self {
      chunk: None,
      runtime_requirements,
    }
  }
  pub fn generate_base_uri(&self, root_output_dir: &str) -> String {
    let options = self.chunk.get_entry_options();
    if let Some(entry_options) = options {
      if let Some(base_uri) = entry_options.base_uri {
        return format!(
          "{} = {};",
          runtime_globals::base_uri,
          &base_uri.toString().unwrap()
        );
      }
    }

    let file_path = if let Some(dir) = root_output_dir.strip_prefix("/") {
      format!("__dirname + {:?}", dir)
    } else {
      String::from("__filename")
    };

    return format!(
      "{} = require(\"url\").pathToFileURL({});",
      runtime_globals::BASE_URI,
      file_path
    );
  }
}

impl RuntimeModule for ReadFileChunkLoadingRuntimeModule {
  fn identifier(&self) -> String {
    "webpack/runtime/read_file_js_chunk_loading".to_string()
  }

  fn generate(&self, compilation: &Compilation) -> BoxSource {
    let with_hmr = self
      .runtime_requirements
      .contains(runtime_globals::HMR_DOWNLOAD_UPDATE_HANDLERS);

    let with_base_uri = self
      .runtime_requirements
      .contains(runtime_globals::BASE_URI);

    let with_external_install_chunk = self
      .runtime_requirements
      .contains(runtime_globals::EXTERNAL_INSTALL_CHUNK);
    let initial_chunks = get_initial_chunk_ids(self.chunk, compilation, chunk_has_js);
    let mut source = ConcatSource::default();

    if with_hmr {
      source.add(RawSource::from(format!(
        "var installedChunks = {} = {} || {};\n",
        runtime_globals::HMR_RUNTIME_STATE_PREFIX,
        runtime_globals::HMR_RUNTIME_STATE_PREFIX,
        &stringify_chunks(&initial_chunks, 0)
      )));
    } else {
      source.add(RawSource::from(format!(
        "var installedChunks = {};\n",
        &stringify_chunks(&initial_chunks, 0)
      )));
    }

    if self
      .runtime_requirements
      .contains(runtime_globals::ON_CHUNKS_LOADED)
    {
      source.add(RawSource::from(include_str!(
        "runtime/require_chunk_loading.js"
      )));
    } else {
      source.add(RawSource::from("// no on chunks loaded"));
    }

    let with_loading = self
      .runtime_requirements
      .contains(runtime_globals::ENSURE_CHUNK_HANDLERS);

    if with_loading || with_external_install_chunk {
      source.add(RawSource::from(include_str!(
        "runtime/require_chunk_loading.js"
      )));
    }

    if with_loading {
      source.add(RawSource::from(
        include_str!("runtime/require_chunk_loading_with_loading.js")
          // TODO
          .replace("JS_MATCHER", "chunkId"),
      ));
    }

    if with_hmr {
      source.add(RawSource::from(
        include_str!("runtime/require_chunk_loading_with_hmr.js").to_string(),
      ));
      source.add(RawSource::from(
        include_str!("runtime/javascript_hot_module_replacement.js").replace("$key$", "jsonp"),
      ));
    }

    if self
      .runtime_requirements
      .contains(runtime_globals::HMR_DOWNLOAD_MANIFEST)
    {
      source.add(RawSource::from(
        include_str!("runtime/require_chunk_loading_with_hmr_manifest.js").to_string(),
      ));
    }

    if self
      .runtime_requirements
      .contains(runtime_globals::ON_CHUNKS_LOADED)
    {
      source.add(RawSource::from(
        include_str!("runtime/require_chunk_loading_with_on_chunk_load.js").to_string(),
      ));
    }

    if with_external_install_chunk {
      source.add(RawSource::from(
        include_str!("runtime/require_chunk_loading_with_external_install_chunk.js").to_string(),
      ));
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
