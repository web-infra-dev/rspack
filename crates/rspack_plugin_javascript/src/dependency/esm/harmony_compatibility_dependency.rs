use rspack_core::{
  DependencyTemplate, InitFragment, InitFragmentStage, RuntimeGlobals, TemplateContext,
  TemplateReplaceSource,
};

// Mark module `__esModule`.
// Add `__webpack_require__.r(__webpack_exports__);`.
#[derive(Debug)]
pub struct HarmonyCompatibilityDependency;

impl DependencyTemplate for HarmonyCompatibilityDependency {
  fn apply(
    &self,
    _source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let TemplateContext {
      runtime_requirements,
      init_fragments,
      compilation,
      module,
      ..
    } = code_generatable_context;
    // TODO __esModule is used
    runtime_requirements.insert(RuntimeGlobals::MAKE_NAMESPACE_OBJECT);
    runtime_requirements.insert(RuntimeGlobals::EXPORTS);
    init_fragments.push(InitFragment::new(
      format!(
        "'use strict';\n{}({});\n", // todo remove strict
        RuntimeGlobals::MAKE_NAMESPACE_OBJECT,
        compilation
          .module_graph
          .module_graph_module_by_identifier(&module.identifier())
          .expect("should have mgm")
          .get_exports_argument()
      ),
      InitFragmentStage::STAGE_HARMONY_EXPORTS,
      None,
    ));

    if compilation.module_graph.is_async(&module.identifier()) {
      runtime_requirements.insert(RuntimeGlobals::MODULE);
      runtime_requirements.insert(RuntimeGlobals::ASYNC_MODULE);
      init_fragments.push(InitFragment::new(
        format!(
          "{}({}, async function (__webpack_handle_async_dependencies__, __webpack_async_result__) {{ try {{\n",
          RuntimeGlobals::ASYNC_MODULE,
          compilation
            .module_graph
            .module_graph_module_by_identifier(&module.identifier())
            .expect("should have mgm")
            .get_module_argument()
        ),
        InitFragmentStage::STAGE_ASYNC_BOUNDARY,
        Some("\n__webpack_async_result__();\n} catch(e) { __webpack_async_result__(e); } });".to_string()),
      ));
    }
  }
}
