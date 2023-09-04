use rspack_core::{
  AsModuleDependency, Dependency, DependencyCategory, DependencyId, DependencyTemplate,
  DependencyType, ExportNameOrSpec, ExportsOfExportsSpec, ExportsSpec, HarmonyExportInitFragment,
  TemplateContext, TemplateReplaceSource,
};
use swc_core::ecma::atoms::JsWord;

// Create _webpack_require__.d(__webpack_exports__, {}) for each export.
#[derive(Debug, Clone)]
pub struct HarmonyExportSpecifierDependency {
  id: DependencyId,
  export: (JsWord, JsWord),
}

impl HarmonyExportSpecifierDependency {
  pub fn new(export: (JsWord, JsWord)) -> Self {
    Self {
      id: DependencyId::new(),
      export,
    }
  }
}

impl Dependency for HarmonyExportSpecifierDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::EsmExportSpecifier
  }

  fn get_exports(&self) -> Option<ExportsSpec> {
    Some(ExportsSpec {
      exports: ExportsOfExportsSpec::Array(vec![ExportNameOrSpec::String(self.export.0.clone())]),
      priority: Some(1),
      can_mangle: None,
      terminal_binding: Some(true),
      from: None,
      dependencies: None,
      hide_export: None,
      exclude_exports: None,
    })
  }
}

impl AsModuleDependency for HarmonyExportSpecifierDependency {}

impl DependencyTemplate for HarmonyExportSpecifierDependency {
  fn apply(
    &self,
    _source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let TemplateContext {
      init_fragments,
      compilation,
      module,
      ..
    } = code_generatable_context;

    let used = if compilation.options.builtins.tree_shaking.is_true() {
      compilation
        .module_graph
        .get_exports_info(&module.identifier())
        .get_used_exports()
        .contains(&self.export.0)
    } else {
      true
    };
    if used {
      init_fragments.push(Box::new(HarmonyExportInitFragment::new(
        self.export.clone(),
      )));
    }
  }
}
