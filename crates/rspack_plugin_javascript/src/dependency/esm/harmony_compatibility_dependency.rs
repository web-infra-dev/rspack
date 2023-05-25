use rspack_core::{
  CodeReplaceSourceDependency, CodeReplaceSourceDependencyContext,
  CodeReplaceSourceDependencyReplaceSource, InitFragment, InitFragmentStage, RuntimeGlobals,
};

// Mark module `__esModule`.
// Add `__webpack_require__.r(__webpack_exports__);`.
#[derive(Debug)]
pub struct HarmonyCompatibilityDependency;

impl CodeReplaceSourceDependency for HarmonyCompatibilityDependency {
  fn apply(
    &self,
    _source: &mut CodeReplaceSourceDependencyReplaceSource,
    code_generatable_context: &mut CodeReplaceSourceDependencyContext,
  ) {
    let CodeReplaceSourceDependencyContext {
      runtime_requirements,
      init_fragments,
      compilation,
      module,
      ..
    } = code_generatable_context;
    // TODO __esModule is used
    runtime_requirements.add(RuntimeGlobals::MAKE_NAMESPACE_OBJECT);
    runtime_requirements.add(RuntimeGlobals::EXPORTS);
    init_fragments.push(InitFragment::new(
      format!(
        "{}({});\n",
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
    // TODO check async module
  }
}
