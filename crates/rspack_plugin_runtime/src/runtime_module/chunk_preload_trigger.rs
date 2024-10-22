use std::hash::BuildHasherDefault;

use cow_utils::CowUtils;
use indexmap::IndexMap;
use rspack_collections::{Identifiable, Identifier};
use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, OriginalSource, RawSource, SourceExt},
  Compilation, RuntimeModule, RuntimeModuleStage,
};
use rustc_hash::FxHasher;

#[impl_runtime_module]
#[derive(Debug)]
pub struct ChunkPreloadTriggerRuntimeModule {
  id: Identifier,
  chunk_map: IndexMap<String, Vec<String>, BuildHasherDefault<FxHasher>>,
}

impl ChunkPreloadTriggerRuntimeModule {
  pub fn new(chunk_map: IndexMap<String, Vec<String>, BuildHasherDefault<FxHasher>>) -> Self {
    Self::with_default(
      Identifier::from("webpack/runtime/chunk_preload_trigger"),
      chunk_map,
    )
  }
}

impl RuntimeModule for ChunkPreloadTriggerRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Trigger
  }

  fn generate(&self, _: &Compilation) -> rspack_error::Result<BoxSource> {
    let generated_code = include_str!("runtime/chunk_preload_trigger.js")
      .cow_replace(
        "$CHUNK_MAP$",
        &serde_json::to_string(&self.chunk_map).expect("invalid json tostring"),
      )
      .to_string();

    let source = if self.source_map_kind.enabled() {
      OriginalSource::new(generated_code, self.identifier().to_string()).boxed()
    } else {
      RawSource::from(generated_code).boxed()
    };
    Ok(source)
  }
}
