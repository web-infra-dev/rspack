use std::hash::BuildHasherDefault;

use indexmap::IndexMap;
use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule, RuntimeModuleStage,
};
use rspack_identifier::Identifier;
use rspack_util::source_map::SourceMapKind;
use rustc_hash::FxHasher;

#[impl_runtime_module]
#[derive(Debug, Eq)]
pub struct ChunkPreloadTriggerRuntimeModule {
  id: Identifier,
  chunk_map: IndexMap<String, Vec<String>, BuildHasherDefault<FxHasher>>,
}

impl ChunkPreloadTriggerRuntimeModule {
  pub fn new(chunk_map: IndexMap<String, Vec<String>, BuildHasherDefault<FxHasher>>) -> Self {
    Self {
      id: Identifier::from("webpack/runtime/chunk_preload_trigger"),
      chunk_map,
      source_map_kind: SourceMapKind::empty(),
      custom_source: None,
    }
  }
}

impl RuntimeModule for ChunkPreloadTriggerRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, _: &Compilation) -> rspack_error::Result<BoxSource> {
    Ok(
      RawSource::from(include_str!("runtime/chunk_preload_trigger.js").replace(
        "$CHUNK_MAP$",
        &serde_json::to_string(&self.chunk_map).expect("invalid json tostring"),
      ))
      .boxed(),
    )
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Trigger
  }
}
