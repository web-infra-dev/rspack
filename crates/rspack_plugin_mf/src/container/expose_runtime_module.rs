use rspack_collections::{Identifiable, Identifier};
use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, OriginalSource, RawSource, SourceExt},
  ChunkUkey, Compilation, RuntimeModule, RuntimeModuleStage, SourceType,
};

use super::container_entry_module::CodeGenerationDataExpose;
use crate::utils::json_stringify;

#[impl_runtime_module]
#[derive(Debug)]
pub struct ExposeRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl ExposeRuntimeModule {
  pub fn new() -> Self {
    Self::with_default(Identifier::from("webpack/runtime/initialize_exposes"), None)
  }
}

impl ExposeRuntimeModule {
  fn find_expose_data<'a>(
    &self,
    chunk_ukey: &ChunkUkey,
    compilation: &'a Compilation,
  ) -> Option<&'a CodeGenerationDataExpose> {
    let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
    let module_graph = compilation.get_module_graph();
    for c in chunk.get_all_initial_chunks(&compilation.chunk_group_by_ukey) {
      let chunk = compilation.chunk_by_ukey.expect_get(&c);
      let modules = compilation
        .chunk_graph
        .get_chunk_modules_iterable_by_source_type(&c, SourceType::Expose, &module_graph);
      for m in modules {
        let code_gen = compilation
          .code_generation_results
          .get(&m.identifier(), Some(&chunk.runtime));
        if let Some(data) = code_gen.data.get::<CodeGenerationDataExpose>() {
          return Some(data);
        };
      }
    }
    None
  }
}

impl RuntimeModule for ExposeRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }

  fn generate(&self, compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    let chunk_ukey = self
      .chunk
      .expect("should have chunk in <ExposeRuntimeModule as RuntimeModule>::generate");
    let Some(data) = self.find_expose_data(&chunk_ukey, compilation) else {
      let generated_code = "";
      let source = if self.source_map_kind.enabled() {
        OriginalSource::new(generated_code, self.identifier().to_string()).boxed()
      } else {
        RawSource::from(generated_code).boxed()
      };
      return Ok(source);
    };
    let module_map = data.module_map.render(compilation);
    let mut generated_code = format!(
      r#"
__webpack_require__.initializeExposesData = {{
  moduleMap: {},
  shareScope: {},
}};
"#,
      module_map,
      json_stringify(&data.share_scope)
    );
    generated_code.push_str("__webpack_require__.getContainer = __webpack_require__.getContainer || function() { throw new Error(\"should have __webpack_require__.getContainer\") };");
    generated_code.push_str("__webpack_require__.initContainer = __webpack_require__.initContainer || function() { throw new Error(\"should have __webpack_require__.initContainer\") };");

    let source = if self.source_map_kind.enabled() {
      OriginalSource::new(generated_code, self.identifier().to_string()).boxed()
    } else {
      RawSource::from(generated_code).boxed()
    };
    Ok(source)
  }
}
