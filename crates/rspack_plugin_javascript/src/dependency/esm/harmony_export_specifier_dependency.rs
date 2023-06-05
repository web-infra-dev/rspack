use std::collections::HashSet;

use rspack_core::{
  tree_shaking::visitor::SymbolRef, CodeReplaceSourceDependency,
  CodeReplaceSourceDependencyContext, CodeReplaceSourceDependencyReplaceSource, InitFragment,
  InitFragmentStage, RuntimeGlobals,
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
    ctxt: &mut CodeReplaceSourceDependencyContext,
  ) {
    let CodeReplaceSourceDependencyContext {
      runtime_requirements,
      init_fragments,
      compilation,
      module,
      ..
    } = ctxt;
    let exports_argument = compilation
      .module_graph
      .module_graph_module_by_identifier(&module.identifier())
      .expect("should have mgm")
      .get_exports_argument();
    runtime_requirements.add(RuntimeGlobals::EXPORTS);

    if !self.exports.is_empty() {
      let module_id = ctxt.module.identifier();
      // TODO: POC
      let used_export = if ctxt.compilation.options.builtins.tree_shaking.is_true() {
        // dbg!(&ctxt.compilation.used_symbol_ref);
        let set = ctxt
          .compilation
          .used_symbol_ref
          .iter()
          .filter_map(|item| match item {
            SymbolRef::Direct(d) if d.uri() == module_id => Some(d.id().atom.to_string()),
            _ => None,
          })
          .collect::<HashSet<_>>();
        Some(set)
      } else {
        None
      };
      dbg!(&module_id);
      dbg!(&used_export);
      runtime_requirements.add(RuntimeGlobals::DEFINE_PROPERTY_GETTERS);
      init_fragments.push(InitFragment::new(
        format!(
          "{}({exports_argument}, {});\n",
          RuntimeGlobals::DEFINE_PROPERTY_GETTERS,
          format_exports(&self.exports, used_export)
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

pub fn format_exports(
  exports: &[(String, String)],
  used_export: Option<HashSet<String>>,
) -> String {
  format!(
    "{{{}}}",
    exports
      .iter()
      .filter_map(|s| {
        if let Some(export_map) = &used_export {
          if !export_map.contains(&s.1) {
            return None;
          }
        }
        Some(format!("'{}': function() {{ return {}; }}", s.0, s.1))
      })
      .collect::<Vec<_>>()
      .join(", ")
  )
}
