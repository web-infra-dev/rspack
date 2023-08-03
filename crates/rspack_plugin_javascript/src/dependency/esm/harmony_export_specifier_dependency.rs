use rspack_core::{
  DependencyTemplate, InitFragment, InitFragmentStage, RuntimeGlobals, TemplateContext,
  TemplateReplaceSource,
};
use swc_core::ecma::atoms::JsWord;

// Create _webpack_require__.d(__webpack_exports__, {}) for each export.
#[derive(Debug)]
pub struct HarmonyExportSpecifierDependency {
  exports: Vec<(JsWord, JsWord)>,
}

impl HarmonyExportSpecifierDependency {
  pub fn new(exports: Vec<(JsWord, JsWord)>) -> Self {
    Self { exports }
  }
}

impl DependencyTemplate for HarmonyExportSpecifierDependency {
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
    let exports_argument = compilation
      .module_graph
      .module_graph_module_by_identifier(&module.identifier())
      .expect("should have mgm")
      .get_exports_argument();
    runtime_requirements.insert(RuntimeGlobals::EXPORTS);

    if !self.exports.is_empty() {
      let used_exports = if compilation.options.builtins.tree_shaking.is_true() {
        Some(
          compilation
            .module_graph
            .get_exports_info(&module.identifier())
            .get_used_exports(),
        )
      } else {
        None
      };
      let exports = self
        .exports
        .clone()
        .into_iter()
        .filter(|s| {
          if let Some(export_map) = &used_exports {
            return export_map.contains(&s.0);
          }
          true
        })
        .collect::<Vec<_>>();
      if !exports.is_empty() {
        runtime_requirements.insert(RuntimeGlobals::DEFINE_PROPERTY_GETTERS);
        init_fragments.push(InitFragment::new(
          format!(
            "{}({exports_argument}, {});\n",
            RuntimeGlobals::DEFINE_PROPERTY_GETTERS,
            format_exports(&exports)
          ),
          InitFragmentStage::StageHarmonyExports,
          None,
        ));
      } else {
        // dbg!(&used_exports);
        // dbg!(&self.exports);
      }
    }
  }
}

pub fn format_exports(exports: &[(JsWord, JsWord)]) -> String {
  format!(
    "{{\n  {}\n}}",
    exports
      .iter()
      .map(|s| format!("'{}': function() {{ return {}; }}", s.0, s.1))
      .collect::<Vec<_>>()
      .join(",\n  ")
  )
}
