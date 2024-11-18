use napi_derive::napi;
use rspack_core::incremental::IncrementalPasses;

#[derive(Debug, Default)]
#[napi(object)]
pub struct RawIncremental {
  pub make: bool,
  pub infer_async_modules: bool,
  pub provided_exports: bool,
  pub dependencies_diagnostics: bool,
  pub build_chunk_graph: bool,
  pub modules_hashes: bool,
  pub modules_codegen: bool,
  pub modules_runtime_requirements: bool,
  pub chunks_runtime_requirements: bool,
  pub chunks_hashes: bool,
  pub chunks_render: bool,
  pub emit_assets: bool,
}

impl From<RawIncremental> for IncrementalPasses {
  fn from(value: RawIncremental) -> Self {
    let mut passes = IncrementalPasses::empty();
    if value.make {
      passes.insert(IncrementalPasses::MAKE);
    }
    if value.infer_async_modules {
      passes.insert(IncrementalPasses::INFER_ASYNC_MODULES);
    }
    if value.provided_exports {
      passes.insert(IncrementalPasses::PROVIDED_EXPORTS);
    }
    if value.dependencies_diagnostics {
      passes.insert(IncrementalPasses::DEPENDENCIES_DIAGNOSTICS);
    }
    if value.build_chunk_graph {
      passes.insert(IncrementalPasses::BUILD_CHUNK_GRAPH);
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
    if value.chunks_render {
      passes.insert(IncrementalPasses::CHUNKS_RENDER);
    }
    if value.emit_assets {
      passes.insert(IncrementalPasses::EMIT_ASSETS);
    }
    passes
  }
}
