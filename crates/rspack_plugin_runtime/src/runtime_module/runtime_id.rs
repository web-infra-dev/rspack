use itertools::Itertools;
use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeTemplate, impl_runtime_module,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct RuntimeIdRuntimeModule {}

impl RuntimeIdRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(runtime_template)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for RuntimeIdRuntimeModule {
  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
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
          .expect("At least one runtime"),
      );

      Ok(format!(
        "{} = {};",
        compilation
          .runtime_template
          .render_runtime_globals(&RuntimeGlobals::RUNTIME_ID),
        serde_json::to_string(&id).expect("Invalid json string")
      ))
    } else {
      unreachable!("should attach chunk for css_loading")
    }
  }
}
