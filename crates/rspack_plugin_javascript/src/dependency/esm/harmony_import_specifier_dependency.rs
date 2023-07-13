use rspack_core::{export_from_import, DependencyId, TemplateContext, TemplateReplaceSource};
use swc_core::ecma::atoms::JsWord;

#[derive(Debug, Clone)]
pub struct HarmonyImportSpecifierDependency {
  shorthand: bool,
  start: u32,
  end: u32,
  // harmony_harmony_import_dependency: &'a HarmonyImportDependency,
  ids: Vec<JsWord>,
  is_call: bool,
}

impl HarmonyImportSpecifierDependency {
  pub fn new(
    shorthand: bool,
    start: u32,
    end: u32,
    // harmony_harmony_import_dependency: &'a HarmonyImportDependency,
    ids: Vec<JsWord>,
    is_call: bool,
  ) -> Self {
    Self {
      shorthand,
      start,
      end,
      // harmony_harmony_import_dependency,
      ids,
      is_call,
    }
  }

  pub fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
    id: &DependencyId,
    request: &str,
    used: bool,
  ) {
    if !used {
      source.replace(
        self.start,
        self.end,
        &format!("/* {} unused */undefined", request),
        None,
      );
      return;
    }

    let import_var = code_generatable_context
      .compilation
      .module_graph
      .get_import_var(&code_generatable_context.module.identifier(), request);

    let export_expr = export_from_import(
      code_generatable_context,
      true,
      import_var,
      self.ids.clone(),
      id,
      self.is_call,
    );
    if self.shorthand {
      source.insert(self.end, format!(": {export_expr}").as_str(), None);
    } else {
      source.replace(self.start, self.end, export_expr.as_str(), None)
    }
  }
}
