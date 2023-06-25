use std::collections::HashSet;

use rspack_core::{
  tree_shaking::visitor::SymbolRef, CodeGeneratableContext, CodeGeneratableDependency,
  CodeGeneratableSource, InitFragment, InitFragmentStage, RuntimeGlobals,
};
use rspack_symbol::{IndirectType, SymbolType, DEFAULT_JS_WORD};
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

impl CodeGeneratableDependency for HarmonyExportSpecifierDependency {
  fn apply(
    &self,
    _source: &mut CodeGeneratableSource,
    code_generatable_context: &mut CodeGeneratableContext,
  ) {
    let CodeGeneratableContext {
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
            SymbolRef::Direct(d) if d.src() == module.identifier() => {
              if *d.ty() == SymbolType::Temp {
                if let Some(key) = self
                  .exports
                  .iter()
                  .find(|e| e.1 == d.id().atom && e.0 != d.id().atom)
                {
                  return Some(&key.0);
                }
              }
              Some(&d.id().atom)
            }
            SymbolRef::Indirect(i) if i.importer == module.identifier() && i.is_reexport() => {
              Some(i.id())
            }
            SymbolRef::Indirect(i) if i.src == module.identifier() => match i.ty {
              // IndirectType::Import(_, _) => Some(i.indirect_id()),
              IndirectType::ImportDefault(_) => Some(&DEFAULT_JS_WORD),
              _ => None,
            },
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
        // dbg!(&used_exports);
        // dbg!(&self.exports);
      }
    }
  }
}

pub fn format_exports(exports: &[(JsWord, JsWord)]) -> String {
  format!(
    "{{{}}}",
    exports
      .iter()
      .map(|s| format!("'{}': function() {{ return {}; }}", s.0, s.1))
      .collect::<Vec<_>>()
      .join(", ")
  )
}
