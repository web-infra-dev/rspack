use rspack_core::{
  CodeReplaceSourceDependency, CodeReplaceSourceDependencyContext,
  CodeReplaceSourceDependencyReplaceSource, InitFragment, InitFragmentStage, RuntimeGlobals,
};

// Create _webpack_require__.d(__webpack_exports__, {}) for each export.
// Exclude re-exports.
#[derive(Debug)]
pub struct HarmonyExportSpecifierDependency {
  exports: Vec<(String, String)>,
  exports_all: Vec<String>,
}

impl HarmonyExportSpecifierDependency {
  pub fn new(exports: Vec<(String, String)>, exports_all: Vec<String>) -> Self {
    Self {
      exports,
      exports_all,
    }
  }
}

impl CodeReplaceSourceDependency for HarmonyExportSpecifierDependency {
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
    let exports_argument = compilation
      .module_graph
      .module_graph_module_by_identifier(&module.identifier())
      .expect("should have mgm")
      .get_exports_argument();
    runtime_requirements.add(RuntimeGlobals::EXPORTS);

    if !self.exports.is_empty() {
      runtime_requirements.add(RuntimeGlobals::DEFINE_PROPERTY_GETTERS);
      init_fragments.push(InitFragment::new(
        format!(
          "{}({exports_argument}, {});\n",
          RuntimeGlobals::DEFINE_PROPERTY_GETTERS,
          format_exports(&self.exports)
        ),
        InitFragmentStage::STAGE_HARMONY_EXPORTS,
        None,
      ));
    }

    // TODO align to webpack
    if !self.exports_all.is_empty() {
      runtime_requirements.add(RuntimeGlobals::EXPORT_STAR);
      self.exports_all.iter().for_each(|all| {
        init_fragments.push(InitFragment::new(
          format!(
            "{}.{}({all}, {exports_argument});\n",
            RuntimeGlobals::REQUIRE,
            RuntimeGlobals::EXPORT_STAR,
          ),
          InitFragmentStage::STAGE_PROVIDES,
          None,
        ));
      });
    }
  }
}

pub fn format_exports(exports: &[(String, String)]) -> String {
  format!(
    "{{{}}}",
    exports
      .iter()
      .map(|s| format!("'{}': function() {{ return {}; }}", s.0, s.1))
      .collect::<Vec<_>>()
      .join(", ")
  )
}
