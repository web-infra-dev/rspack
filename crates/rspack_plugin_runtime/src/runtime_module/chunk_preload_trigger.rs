use std::hash::BuildHasherDefault;

use indexmap::IndexMap;
use rspack_cacheable::with::AsMap;
use rspack_collections::Identifier;
use rspack_core::{
  Compilation, RuntimeModule, RuntimeModuleStage, chunk_graph_chunk::ChunkId, impl_runtime_module,
};
use rustc_hash::FxHasher;

#[impl_runtime_module]
#[derive(Debug)]
pub struct ChunkPreloadTriggerRuntimeModule {
  id: Identifier,
  #[cacheable(with=AsMap)]
  chunk_map: IndexMap<ChunkId, Vec<ChunkId>, BuildHasherDefault<FxHasher>>,
}

impl ChunkPreloadTriggerRuntimeModule {
  pub fn new(chunk_map: IndexMap<ChunkId, Vec<ChunkId>, BuildHasherDefault<FxHasher>>) -> Self {
    Self::with_default(
      Identifier::from("webpack/runtime/chunk_preload_trigger"),
      chunk_map,
    )
  }
}

#[async_trait::async_trait]
impl RuntimeModule for ChunkPreloadTriggerRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      include_str!("runtime/chunk_preload_trigger.ejs").to_string(),
    )]
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    let source = compilation.runtime_template.render(
      &self.id,
      Some(serde_json::json!({
        "_chunk_map": &self.chunk_map,
      })),
    )?;

    Ok(source)
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Trigger
  }
}
