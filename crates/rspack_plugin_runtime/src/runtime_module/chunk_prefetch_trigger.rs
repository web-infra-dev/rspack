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
pub struct ChunkPrefetchTriggerRuntimeModule {
  id: Identifier,
  chunk_map: IndexMap<String, Vec<String>, BuildHasherDefault<FxHasher>>,
}

impl ChunkPrefetchTriggerRuntimeModule {
  pub fn new(chunk_map: IndexMap<String, Vec<String>, BuildHasherDefault<FxHasher>>) -> Self {
    Self {
      id: Identifier::from("webpack/runtime/chunk_prefetch_trigger"),
      chunk_map,
      source_map_kind: SourceMapKind::None,
      custom_source: None,
    }
  }
}

impl RuntimeModule for ChunkPrefetchTriggerRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, _: &Compilation) -> rspack_error::Result<BoxSource> {
    Ok(
      RawSource::from(include_str!("runtime/chunk_prefetch_trigger.js").replace(
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
