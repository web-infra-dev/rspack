use itertools::Itertools;
use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleStage,
};
use rspack_identifier::Identifier;

#[impl_runtime_module]
#[derive(Debug)]
pub struct ChunkPrefetchStartupRuntimeModule {
  id: Identifier,
  startup_chunks: Vec<(Vec<ChunkUkey>, Vec<ChunkUkey>)>,
  chunk: Option<ChunkUkey>,
}

impl ChunkPrefetchStartupRuntimeModule {
  pub fn new(startup_chunks: Vec<(Vec<ChunkUkey>, Vec<ChunkUkey>)>) -> Self {
    Self::with_default(
      Identifier::from("webpack/runtime/chunk_prefetch_startup"),
      startup_chunks,
      None,
    )
  }
}

impl RuntimeModule for ChunkPrefetchStartupRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }

  fn generate(&self, compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    let chunk_ukey = self.chunk.expect("chunk do not attached");
    Ok(
      RawSource::from(
        self
          .startup_chunks
          .iter()
          .map(|(group_chunks, child_chunks)| {
            let group_chunk_ids = group_chunks
              .iter()
              .filter_map(|c| {
                if c.to_owned().eq(&chunk_ukey) {
                  compilation.chunk_by_ukey.expect_get(c).id.to_owned()
                } else {
                  None
                }
              })
              .collect_vec();

            let child_chunk_ids = child_chunks
              .iter()
              .filter_map(|c| compilation.chunk_by_ukey.expect_get(c).id.to_owned())
              .collect_vec();

            let body = match child_chunks.len() {
              x if x < 3 => child_chunk_ids
                .iter()
                .map(|id| {
                  format!(
                    "{}({});",
                    RuntimeGlobals::PREFETCH_CHUNK,
                    serde_json::to_string(&id).expect("invalid json tostring")
                  )
                })
                .join("\n"),
              _ => {
                format!(
                  "{}.map({})",
                  serde_json::to_string(&child_chunk_ids).expect("invalid json tostring"),
                  RuntimeGlobals::PREFETCH_CHUNK
                )
              }
            };

            format!(
              r#"
            {}(0, {}, function() {{
              {}
            }}, 5);
            "#,
              RuntimeGlobals::ON_CHUNKS_LOADED,
              serde_json::to_string(&group_chunk_ids).expect("invalid json tostring"),
              body
            )
          })
          .join("\n"),
      )
      .boxed(),
    )
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Trigger
  }
}
