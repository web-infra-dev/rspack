use itertools::Itertools;
use rspack_core::{
  RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext, RuntimeTemplate, impl_runtime_module,
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
  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let compilation = context.compilation;
    if let Some(chunk_ukey) = self.chunk {
      let chunk = compilation
        .build_chunk_graph_artifact
        .chunk_by_ukey
        .expect_get(&chunk_ukey);

      let runtime = chunk.runtime();

      if runtime.len() > 1 {
        panic!("RuntimeIdRuntimeModule must be in a single runtime");
      }

      let id = compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .get_runtime_id(
          runtime
            .iter()
            .collect_vec()
            .first()
            .expect("At least one runtime"),
        );

      Ok(format!(
        "{} = {};",
        context
          .runtime_template
          .render_runtime_globals(&RuntimeGlobals::RUNTIME_ID),
        serde_json::to_string(&id).expect("Invalid json string")
      ))
    } else {
      unreachable!("should attach chunk for css_loading")
    }
  }
}
