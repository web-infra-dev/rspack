use rspack_core::{
  basic_function, impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleStage, SourceType,
};
use rspack_identifier::Identifier;
use rspack_util::source_map::SourceMapKind;

use crate::utils::json_stringify;

#[impl_runtime_module]
#[derive(Debug, Eq)]
pub struct ExposeRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
  enhanced: bool,
}

impl ExposeRuntimeModule {
  pub fn new(enhanced: bool) -> Self {
    Self {
      id: Identifier::from("webpack/runtime/initialize_exposes"),
      chunk: None,
      enhanced,
      source_map_kind: SourceMapKind::None,
      custom_source: None,
    }
  }
}

impl ExposeRuntimeModule {
  fn find_expose_data<'a>(
    &self,
    chunk_ukey: &ChunkUkey,
    compilation: &'a Compilation,
  ) -> Option<&'a CodeGenerationDataExpose> {
    let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
    for c in chunk.get_all_initial_chunks(&compilation.chunk_group_by_ukey) {
      let chunk = compilation.chunk_by_ukey.expect_get(&c);
      let modules = compilation
        .chunk_graph
        .get_chunk_modules_iterable_by_source_type(
          &c,
          SourceType::Expose,
          compilation.get_module_graph(),
        );
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

  fn generate(&self, compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    let chunk_ukey = self
      .chunk
      .expect("should have chunk in <ExposeRuntimeModule as RuntimeModule>::generate");
    let Some(data) = self.find_expose_data(&chunk_ukey, compilation) else {
      return Ok(RawSource::from("").boxed());
    };
    let module_map = data
      .module_map
      .iter()
      .map(|(name, factory)| format!("{}: {},", json_stringify(name), basic_function("", factory)))
      .collect::<Vec<_>>()
      .join("\n");
    let mut source = format!(
      r#"
__webpack_require__.initializeExposesData = {{
  moduleMap: {{
{}
  }},
  shareScope: {},
}};
"#,
      module_map,
      json_stringify(&data.share_scope)
    );
    if self.enhanced {
      source += "__webpack_require__.getContainer = function() { throw new Error(\"should have __webpack_require__.getContainer\") };";
      source += "__webpack_require__.initContainer = function() { throw new Error(\"should have __webpack_require__.initContainer\") };";
    } else {
      source += &format!(
        r#"
__webpack_require__.getContainer = function(module, getScope) {{
  var moduleMap = __webpack_require__.initializeExposesData.moduleMap;
  {current_remote_get_scope} = getScope;
  getScope = (
    {has_own_property}(moduleMap, module)
      ? moduleMap[module]()
      : Promise.resolve().then(() => {{
        throw new Error('Module "' + module + '" does not exist in container.');
      }})
  );
  {current_remote_get_scope} = undefined;
  return getScope;
}}"#,
        current_remote_get_scope = RuntimeGlobals::CURRENT_REMOTE_GET_SCOPE,
        has_own_property = RuntimeGlobals::HAS_OWN_PROPERTY
      );
      source += &format!(
        r#"
__webpack_require__.initContainer = function(shareScope, initScope) {{
  if (!{share_scope_map}) return;
  var name = __webpack_require__.initializeExposesData.shareScope;
  var oldScope = {share_scope_map}[name];
  if(oldScope && oldScope !== shareScope) throw new Error("Container initialization failed as it has already been initialized with a different share scope");
  {share_scope_map}[name] = shareScope;
  return {initialize_sharing}(name, initScope);
}}"#,
        share_scope_map = RuntimeGlobals::SHARE_SCOPE_MAP,
        initialize_sharing = RuntimeGlobals::INITIALIZE_SHARING,
      );
    }
    Ok(RawSource::from(source).boxed())
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }
}

#[derive(Debug, Clone)]
pub struct CodeGenerationDataExpose {
  pub module_map: Vec<(String, String)>,
  pub share_scope: String,
}
