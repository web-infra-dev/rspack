use itertools::Itertools;
use rspack_collections::Identifier;
use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawStringSource, SourceExt},
  ChunkUkey, Compilation, RuntimeModule,
};

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

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      include_str!("runtime/runtime_module.ejs").to_string(),
    )]
  }

  fn generate(&self, compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    if let Some(chunk_ukey) = self.chunk {
      let chunk = compilation.chunk_by_ukey.expect_get(&chunk_ukey);

      let runtime = chunk.runtime();

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

      let source = compilation.runtime_template.render(
        &self.id,
        Some(serde_json::json!({
          "ID": serde_json::to_string(&id).expect("Invalid json string"),
        })),
      )?;

      Ok(RawStringSource::from(source).boxed())
    } else {
      unreachable!("should attach chunk for css_loading")
    }
  }
}
