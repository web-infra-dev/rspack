use rspack_core::{
  export_from_import, get_import_var, CodeReplaceSourceDependencyContext,
  CodeReplaceSourceDependencyReplaceSource, DependencyId, InitFragment, InitFragmentStage,
  RuntimeGlobals,
};

use super::format_exports;

#[derive(Debug, Clone)]
pub struct HarmonyImportSpecifierDependency {
  shorthand: bool,
  start: u32,
  end: u32,
  // harmony_harmony_import_dependency: &'a HarmonyImportDependency,
  ids: Option<String>,
  export: Option<String>,
}

impl HarmonyImportSpecifierDependency {
  pub fn new(
    shorthand: bool,
    start: u32,
    end: u32,
    // harmony_harmony_import_dependency: &'a HarmonyImportDependency,
    ids: Option<String>,
    export: Option<String>,
  ) -> Self {
    Self {
      shorthand,
      start,
      end,
      // harmony_harmony_import_dependency,
      ids,
      export,
    }
  }

  pub fn apply(
    &self,
    source: &mut CodeReplaceSourceDependencyReplaceSource,
    code_generatable_context: &mut CodeReplaceSourceDependencyContext,
    id: &DependencyId,
    request: &str,
  ) {
    let import_var = get_import_var(request);

    let export_expr = export_from_import(
      code_generatable_context,
      true,
      import_var,
      self
        .ids
        .as_ref()
        .map(|i| vec![i.clone()])
        .unwrap_or_default(),
      id,
      false,
    );

    if let Some(export) = &self.export {
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
      runtime_requirements.add(RuntimeGlobals::DEFINE_PROPERTY_GETTERS);
      init_fragments.push(InitFragment::new(
        format!(
          "{}({exports_argument}, {});\n",
          RuntimeGlobals::DEFINE_PROPERTY_GETTERS,
          format_exports(&[(export.to_string(), export_expr)])
        ),
        InitFragmentStage::STAGE_HARMONY_EXPORTS,
        None,
      ));
    } else if self.shorthand {
      source.insert(self.end, format!(": {export_expr}").as_str(), None);
    } else {
      source.replace(self.start, self.end, export_expr.as_str(), None)
    }
  }
}
