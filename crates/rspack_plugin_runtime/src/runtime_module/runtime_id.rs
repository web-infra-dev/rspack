use itertools::Itertools;
use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule,
};
use rspack_identifier::Identifier;

#[impl_runtime_module]
#[derive(Debug)]
pub struct RuntimeIdRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl Default for RuntimeIdRuntimeModule {
  fn default() -> Self {
    Self::with_default(Identifier::from("webpack/runtime/runtime_id"), None)
  }
}

impl RuntimeModule for RuntimeIdRuntimeModule {
  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }

  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    if let Some(chunk_ukey) = self.chunk {
      let chunk = compilation.chunk_by_ukey.expect_get(&chunk_ukey);

      let runtime = &chunk.runtime;

      if runtime.len() > 1 {
        panic!("RuntimeIdRuntimeModule must be in a single runtime");
      }

      let id = compilation.chunk_graph.get_runtime_id(
        runtime
          .iter()
          .collect_vec()
          .first()
          .expect("At least one runtime")
          .to_string(),
      );

      Ok(
        RawSource::from(format!(
          "{} = {};",
          RuntimeGlobals::RUNTIME_ID,
          serde_json::to_string(&id).expect("Invalid json string")
        ))
        .boxed(),
      )
    } else {
      unreachable!("should attach chunk for css_loading")
    }
  }
}
