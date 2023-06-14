use rspack_core::{
  export_from_import, get_import_var, CodeReplaceSourceDependencyContext,
  CodeReplaceSourceDependencyReplaceSource, DependencyId,
};
use swc_core::ecma::atoms::JsWord;

#[derive(Debug, Clone)]
pub struct HarmonyImportSpecifierDependency {
  shorthand: bool,
  start: u32,
  end: u32,
  // harmony_harmony_import_dependency: &'a HarmonyImportDependency,
  ids: Option<JsWord>,
}

impl HarmonyImportSpecifierDependency {
  pub fn new(
    shorthand: bool,
    start: u32,
    end: u32,
    // harmony_harmony_import_dependency: &'a HarmonyImportDependency,
    ids: Option<JsWord>,
  ) -> Self {
    Self {
      shorthand,
      start,
      end,
      // harmony_harmony_import_dependency,
      ids,
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
    if self.shorthand {
      source.insert(self.end, format!(": {export_expr}").as_str(), None);
    } else {
      source.replace(self.start, self.end, export_expr.as_str(), None)
    }
  }
}
