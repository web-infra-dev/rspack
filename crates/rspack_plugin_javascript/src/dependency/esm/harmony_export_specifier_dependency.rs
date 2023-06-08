use std::collections::HashSet;

use rspack_core::{
  tree_shaking::visitor::SymbolRef, CodeReplaceSourceDependency,
  CodeReplaceSourceDependencyContext, CodeReplaceSourceDependencyReplaceSource, DependencyType,
  InitFragment, InitFragmentStage, RuntimeGlobals,
};

// Create _webpack_require__.d(__webpack_exports__, {}) for each export.
// Exclude re-exports.
#[derive(Debug)]
pub struct HarmonyExportSpecifierDependency {
  exports: Vec<(String, String)>,
}

impl HarmonyExportSpecifierDependency {
  pub fn new(exports: Vec<(String, String)>) -> Self {
    Self { exports }
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
      let used_exports = if compilation.options.builtins.tree_shaking.is_true() {
        let set = compilation
          .used_symbol_ref
          .iter()
          .filter_map(|item| match item {
            SymbolRef::Direct(d) if d.uri() == module.identifier() => Some(d.id().atom.to_string()),
            _ => None,
          })
          .collect::<HashSet<_>>();
        Some(set)
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
        runtime_requirements.add(RuntimeGlobals::DEFINE_PROPERTY_GETTERS);
        init_fragments.push(InitFragment::new(
          format!(
            "{}({exports_argument}, {});\n",
            RuntimeGlobals::DEFINE_PROPERTY_GETTERS,
            format_exports(&exports)
          ),
          InitFragmentStage::STAGE_HARMONY_EXPORTS,
          None,
        ));
      } else {
        dbg!(&used_exports);
        dbg!(&self.exports);
      }
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
