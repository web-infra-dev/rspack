use napi_derive::napi;
use rspack_core::incremental::{IncrementalOptions, IncrementalPasses};

#[derive(Debug, Default)]
#[napi(object)]
pub struct RawIncremental {
  pub silent: bool,
  // passes
  pub build_module_graph: bool,
  pub finish_modules: bool,
  pub optimize_dependencies: bool,
  pub build_chunk_graph: bool,
  pub optimize_chunk_modules: bool,
  pub module_ids: bool,
  pub chunk_ids: bool,
  pub modules_hashes: bool,
  pub modules_codegen: bool,
  pub modules_runtime_requirements: bool,
  pub chunks_runtime_requirements: bool,
  pub chunks_hashes: bool,
  pub chunk_asset: bool,
  pub emit_assets: bool,
}

impl From<RawIncremental> for IncrementalOptions {
  fn from(value: RawIncremental) -> Self {
    let mut passes = IncrementalPasses::empty();
    if value.build_module_graph {
      passes.insert(IncrementalPasses::BUILD_MODULE_GRAPH);
    }
    if value.finish_modules {
      passes.insert(IncrementalPasses::FINISH_MODULES);
    }
    if value.optimize_dependencies {
      passes.insert(IncrementalPasses::OPTIMIZE_DEPENDENCIES);
    }
    if value.build_chunk_graph {
      passes.insert(IncrementalPasses::BUILD_CHUNK_GRAPH);
    }
    if value.optimize_chunk_modules {
      passes.insert(IncrementalPasses::OPTIMIZE_CHUNK_MODULES);
    }
    if value.module_ids {
      passes.insert(IncrementalPasses::MODULE_IDS);
    }
    if value.chunk_ids {
      passes.insert(IncrementalPasses::CHUNK_IDS);
    }
    if value.modules_hashes {
      passes.insert(IncrementalPasses::MODULES_HASHES);
    }
    if value.modules_codegen {
      passes.insert(IncrementalPasses::MODULES_CODEGEN);
    }
    if value.modules_runtime_requirements {
      passes.insert(IncrementalPasses::MODULES_RUNTIME_REQUIREMENTS);
    }
    if value.chunks_runtime_requirements {
      passes.insert(IncrementalPasses::CHUNKS_RUNTIME_REQUIREMENTS);
    }
    if value.chunks_hashes {
      passes.insert(IncrementalPasses::CHUNKS_HASHES);
    }
    if value.chunk_asset {
      passes.insert(IncrementalPasses::CHUNK_ASSET);
    }
    if value.emit_assets {
      passes.insert(IncrementalPasses::EMIT_ASSETS);
    }
    Self {
      silent: value.silent,
      passes,
    }
  }
}
