use rspack_core::{
  ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext,
  RuntimeModuleStage, RuntimeTemplate, SourceType, impl_runtime_module,
};

use super::container_entry_module::CodeGenerationDataExpose;
use crate::utils::json_stringify;

#[impl_runtime_module]
#[derive(Debug)]
pub struct ExposeRuntimeModule {}

impl ExposeRuntimeModule {
  #[allow(clippy::new_without_default)]
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_name(runtime_template, "initialize_exposes")
  }
}

impl ExposeRuntimeModule {
  fn find_expose_data<'a>(
    &self,
    chunk_ukey: &ChunkUkey,
    compilation: &'a Compilation,
  ) -> Option<&'a CodeGenerationDataExpose> {
    let chunk = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .expect_get(chunk_ukey);
    let module_graph = compilation.get_module_graph();
    for c in
      chunk.get_all_initial_chunks(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey)
    {
      let chunk = compilation
        .build_chunk_graph_artifact
        .chunk_by_ukey
        .expect_get(&c);
      let modules = compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .get_chunk_modules_identifier_by_source_type(&c, SourceType::Expose, module_graph);
      for m in modules {
        let code_gen = compilation
          .code_generation_results
          .get(&m, Some(chunk.runtime()));
        if let Some(data) = code_gen.data.get::<CodeGenerationDataExpose>() {
          return Some(data);
        };
      }
    }
    None
  }
}

#[async_trait::async_trait]
impl RuntimeModule for ExposeRuntimeModule {
  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let compilation = context.compilation;
    let chunk_ukey = self
      .chunk
      .expect("should have chunk in <ExposeRuntimeModule as RuntimeModule>::generate");
    let Some(data) = self.find_expose_data(&chunk_ukey, compilation) else {
      return Ok(String::new());
    };
    let mut runtime_template = compilation.runtime_template.create_module_code_template();
    let module_map = data.module_map.render(&mut runtime_template);
    let require_name = runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE);
    let mut source = format!(
      r#"
    {require_name}.initializeExposesData = {{
  moduleMap: {},
  shareScope: {},
}};
"#,
      module_map,
      json_stringify(&data.share_scope)
    );
    source += &format!(
      "{require_name}.getContainer = {require_name}.getContainer || function() {{ throw new Error(\"should have {require_name}.getContainer\") }};",
    );
    source += &format!(
      "{require_name}.initContainer = {require_name}.initContainer || function() {{ throw new Error(\"should have {require_name}.initContainer\") }};",
    );
    Ok(source)
  }
}
