use rspack_core::{
  AsModuleDependency, Dependency, DependencyCategory, DependencyId, DependencyTemplate,
  DependencyType, ExportNameOrSpec, ExportsOfExportsSpec, ExportsSpec, HarmonyExportInitFragment,
  TemplateContext, TemplateReplaceSource, UsedName,
};
use swc_core::ecma::atoms::JsWord;

// Create _webpack_require__.d(__webpack_exports__, {}) for each export.
#[derive(Debug, Clone)]
pub struct HarmonyExportSpecifierDependency {
  id: DependencyId,
  name: JsWord,
  value: JsWord, // id
}

impl HarmonyExportSpecifierDependency {
  pub fn new(name: JsWord, value: JsWord) -> Self {
    Self {
      id: DependencyId::new(),
      name,
      value,
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
      exports: ExportsOfExportsSpec::Array(vec![ExportNameOrSpec::String(self.name.clone())]),
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
        .old_get_used_exports()
        .contains(&self.name)
    } else if compilation.options.is_new_tree_shaking() {
      let exports_info_id = compilation
        .module_graph
        .get_exports_info(&module.identifier())
        .id;
      let used_name = exports_info_id.get_used_name(
        &compilation.module_graph,
        None,
        UsedName::Str(self.name.clone()),
      );
      // dbg!(&used_name);
      used_name
        .map(|item| match item {
          UsedName::Str(name) => name == self.name,
          UsedName::Vec(vec) => vec.contains(&self.name),
        })
        .unwrap_or_default()
    } else {
      true
    };
    if used {
      init_fragments.push(Box::new(HarmonyExportInitFragment::new((
        self.name.clone(),
        self.value.clone(),
      ))));
    }
  }
}
