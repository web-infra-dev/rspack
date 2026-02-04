use std::sync::LazyLock;

use itertools::Itertools;
use rspack_core::{
  ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleStage, RuntimeTemplate,
  impl_runtime_module,
};

use crate::extract_runtime_globals_from_ejs;

static CHUNK_PREFETCH_STARTUP_TEMPLATE: &str = include_str!("runtime/chunk_prefetch_startup.ejs");
static CHUNK_PREFETCH_STARTUP_RUNTIME_REQUIREMENTS: LazyLock<RuntimeGlobals> =
  LazyLock::new(|| extract_runtime_globals_from_ejs(CHUNK_PREFETCH_STARTUP_TEMPLATE));

#[impl_runtime_module]
#[derive(Debug)]
pub struct ChunkPrefetchStartupRuntimeModule {
  startup_chunks: Vec<(Vec<ChunkUkey>, Vec<ChunkUkey>)>,
}

impl ChunkPrefetchStartupRuntimeModule {
  pub fn new(
    runtime_template: &RuntimeTemplate,
    startup_chunks: Vec<(Vec<ChunkUkey>, Vec<ChunkUkey>)>,
  ) -> Self {
    Self::with_default(runtime_template, startup_chunks)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for ChunkPrefetchStartupRuntimeModule {
  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      CHUNK_PREFETCH_STARTUP_TEMPLATE.to_string(),
    )]
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    let chunk_ukey = self.chunk.expect("chunk do not attached");

    let source = self
      .startup_chunks
      .iter()
      .map(|(group_chunks, child_chunks)| {
        let group_chunk_ids = group_chunks
          .iter()
          .filter_map(|c| {
            if c.to_owned().eq(&chunk_ukey) {
              compilation
                .chunk_by_ukey
                .expect_get(c)
                .id()
            } else {
              None
            }
          })
          .collect_vec();

        let child_chunk_ids = child_chunks
          .iter()
          .filter_map(|c| {
            compilation
              .chunk_by_ukey
              .expect_get(c)
              .id()
          })
          .collect_vec();

        let source = compilation.runtime_template.render(
          &self.id,
          Some(serde_json::json!({
            "_chunk_ids": serde_json::to_string(&group_chunk_ids).expect("invalid json tostring"),
            "_child_chunk_ids": serde_json::to_string(&child_chunk_ids).expect("invalid json tostring"),
          })),
        )?;

        Ok(source)
      })
      .collect::<rspack_error::Result<Vec<String>>>()?
      .join("\n");

    Ok(source)
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Trigger
  }

  fn additional_runtime_requirements(&self, _compilation: &Compilation) -> RuntimeGlobals {
    *CHUNK_PREFETCH_STARTUP_RUNTIME_REQUIREMENTS
  }
}
