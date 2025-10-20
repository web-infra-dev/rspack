use rspack_collections::Identifier;
use rspack_core::{
  ChunkUkey, Compilation, RuntimeModule, RuntimeModuleStage, SourceType, impl_runtime_module,
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
  #[allow(clippy::new_without_default)]
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
    let mut visited = rustc_hash::FxHashSet::default();
    let mut chunks_to_visit = Vec::new();
    chunks_to_visit.push(*chunk_ukey);
    for c in chunk.get_all_initial_chunks(&compilation.chunk_group_by_ukey) {
      chunks_to_visit.push(c);
    }
    for c in chunk.get_all_referenced_chunks(&compilation.chunk_group_by_ukey) {
      chunks_to_visit.push(c);
    }
    let module_graph = compilation.get_module_graph();
    for c in chunks_to_visit {
      if !visited.insert(c) {
        continue;
      }
      let chunk = compilation.chunk_by_ukey.expect_get(&c);
      let modules = compilation
        .chunk_graph
        .get_chunk_modules_identifier_by_source_type(&c, SourceType::Expose, &module_graph);
      for m in modules {
        let code_gen = compilation
          .code_generation_results
          .get(&m, Some(chunk.runtime()));
        if let Some(data) = code_gen.data.get::<CodeGenerationDataExpose>() {
          return Some(data);
        }
      }
    }
    None
  }
}

#[async_trait::async_trait]
impl RuntimeModule for ExposeRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    let chunk_ukey = self
      .chunk
      .expect("should have chunk in <ExposeRuntimeModule as RuntimeModule>::generate");
    let Some(data) = self.find_expose_data(&chunk_ukey, compilation) else {
      return Ok("".to_string());
    };
    let module_map = data.module_map.render(compilation);
    let mut source = format!(
      r#"
__webpack_require__.initializeExposesData = {{
  moduleMap: {},
  shareScope: {},
}};
"#,
      module_map,
      json_stringify(&data.share_scope)
    );
    source += r#"
var __webpack_require__getContainer = __webpack_require__.getContainer;
var __webpack_require__initContainer = __webpack_require__.initContainer;
var __webpack_require__initializeExposesData = __webpack_require__.initializeExposesData;
var hasOwnProperty = Object.prototype.hasOwnProperty;
if (typeof __webpack_require__getContainer !== "function") {
	__webpack_require__.getContainer = function(module, getScope) {
		var moduleMap = __webpack_require__initializeExposesData.moduleMap;
		__webpack_require__.R = getScope;
		var promise = hasOwnProperty.call(moduleMap, module)
			? moduleMap[module]()
			: Promise.resolve().then(function() {
					throw new Error('Module "' + module + '" does not exist in container.');
			  });
		__webpack_require__.R = undefined;
		return promise;
	};
}
if (typeof __webpack_require__initContainer !== "function") {
	__webpack_require__.initContainer = function(shareScope, initScope) {
		if (!__webpack_require__.S) return;
		var name = __webpack_require__initializeExposesData.shareScope;
		var oldScope = __webpack_require__.S[name];
		if (oldScope && oldScope !== shareScope) throw new Error("Container initialization failed as it has already been initialized with a different share scope");
		__webpack_require__.S[name] = shareScope;
		return __webpack_require__.I(name, initScope);
	};
}
"#;
    Ok(source)
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }
}
