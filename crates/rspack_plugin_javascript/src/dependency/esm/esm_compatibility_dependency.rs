use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsDependency, Compilation, DependencyTemplate, InitFragmentKey, InitFragmentStage, ModuleGraph,
  NormalInitFragment, RuntimeGlobals, RuntimeSpec, TemplateContext, TemplateReplaceSource,
  UsageState,
};
use swc_core::atoms::Atom;

// Mark module `__esModule`.
// Add `__webpack_require__.r(__webpack_exports__);`.
#[cacheable]
#[derive(Debug, Clone)]
pub struct ESMCompatibilityDependency;

#[cacheable_dyn]
impl DependencyTemplate for ESMCompatibilityDependency {
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
      runtime,
      concatenation_scope,
      ..
    } = code_generatable_context;
    if concatenation_scope.is_some() {
      return;
    }
    let module_graph = compilation.get_module_graph();
    let module = module_graph
      .module_by_identifier(&module.identifier())
      .expect("should have mgm");
    let exports_info = module_graph.get_exports_info(&module.identifier());
    if !matches!(
      exports_info
        .get_read_only_export_info(&module_graph, &Atom::from("__esModule"),)
        .get_used(&module_graph, *runtime),
      UsageState::Unused
    ) {
      runtime_requirements.insert(RuntimeGlobals::MAKE_NAMESPACE_OBJECT);
      runtime_requirements.insert(RuntimeGlobals::EXPORTS);
      init_fragments.push(Box::new(NormalInitFragment::new(
        format!(
          "{}({});\n",
          RuntimeGlobals::MAKE_NAMESPACE_OBJECT,
          module.get_exports_argument()
        ),
        InitFragmentStage::StageESMExports,
        0,
        InitFragmentKey::ESMCompatibility,
        None,
      )));
    }

    if ModuleGraph::is_async(compilation, &module.identifier()) {
      runtime_requirements.insert(RuntimeGlobals::MODULE);
      runtime_requirements.insert(RuntimeGlobals::ASYNC_MODULE);
      init_fragments.push(Box::new(NormalInitFragment::new(
        format!(
          "{}({}, async function (__webpack_handle_async_dependencies__, __webpack_async_result__) {{ try {{\n",
          RuntimeGlobals::ASYNC_MODULE,
          module_graph
            .module_by_identifier(&module.identifier())
            .expect("should have mgm")
            .get_module_argument()
        ),
        InitFragmentStage::StageAsyncBoundary,
        0,
        InitFragmentKey::unique(),
        Some(format!("\n__webpack_async_result__();\n}} catch(e) {{ __webpack_async_result__(e); }} }}{});", if module.build_meta().has_top_level_await { ", 1" } else { "" })),
      )));
    }
  }

  fn dependency_id(&self) -> Option<rspack_core::DependencyId> {
    None
  }

  fn update_hash(
    &self,
    _hasher: &mut dyn std::hash::Hasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
  }
}
impl AsDependency for ESMCompatibilityDependency {}
