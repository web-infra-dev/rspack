use rspack_core::{
  export_from_import, tree_shaking::visitor::SymbolRef, Compilation, DependencyId,
  DependencyTemplate, Module, ModuleGraphModule, TemplateContext, TemplateReplaceSource,
};
use rspack_symbol::IndirectTopLevelSymbol;
use swc_core::ecma::atoms::JsWord;

use super::Specifier;

#[derive(Debug, Clone)]
pub struct HarmonyImportSpecifierDependency {
  id: DependencyId,
  request: JsWord,
  shorthand: bool,
  start: u32,
  end: u32,
  ids: Vec<JsWord>,
  is_call: bool,
  specifier: Specifier,
}

impl HarmonyImportSpecifierDependency {
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    id: DependencyId,
    request: JsWord,
    shorthand: bool,
    start: u32,
    end: u32,
    ids: Vec<JsWord>,
    is_call: bool,
    specifier: Specifier,
  ) -> Self {
    Self {
      id,
      request,
      shorthand,
      start,
      end,
      ids,
      is_call,
      specifier,
    }
  }

  // TODO move export_info
  pub fn check_used(
    &self,
    module: &dyn Module,
    reference_mgm: &ModuleGraphModule,
    compilation: &Compilation,
  ) -> bool {
    if compilation.options.builtins.tree_shaking.is_false() {
      return true;
    }
    if !compilation
      .include_module_ids
      .contains(&reference_mgm.module_identifier)
    {
      return false;
    }

    if !reference_mgm.module_type.is_js_like() {
      return true;
    }

    match &self.specifier {
      Specifier::Namespace(_) => true,
      Specifier::Default(local) => {
        let symbol = SymbolRef::Indirect(IndirectTopLevelSymbol {
          src: reference_mgm.module_identifier,
          ty: rspack_symbol::IndirectType::ImportDefault(local.clone()),
          importer: module.identifier(),
        });
        compilation.used_symbol_ref.contains(&symbol)
      }
      Specifier::Named(local, imported) => {
        let symbol = SymbolRef::Indirect(IndirectTopLevelSymbol {
          src: reference_mgm.module_identifier,
          ty: rspack_symbol::IndirectType::Import(local.clone(), imported.clone()),
          importer: module.identifier(),
        });
        compilation.used_symbol_ref.contains(&symbol)
      }
    }
  }
}

impl DependencyTemplate for HarmonyImportSpecifierDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let TemplateContext {
      compilation,
      module,
      ..
    } = code_generatable_context;

    let reference_mgm = compilation
      .module_graph
      .module_graph_module_by_dependency_id(&self.id)
      .expect("should have ref module");

    let used = self.check_used(*module, reference_mgm, compilation);

    if !used {
      // TODO do this by PureExpressionDependency.
      source.replace(
        self.start,
        self.end,
        &format!("/* \"{}\" unused */null", self.request),
        None,
      );
      return;
    }

    let import_var = code_generatable_context
      .compilation
      .module_graph
      .get_import_var(&code_generatable_context.module.identifier(), &self.request);

    let export_expr = export_from_import(
      code_generatable_context,
      true,
      import_var,
      self.ids.clone(),
      &self.id,
      self.is_call,
    );
    if self.shorthand {
      source.insert(self.end, format!(": {export_expr}").as_str(), None);
    } else {
      source.replace(self.start, self.end, export_expr.as_str(), None)
    }
  }
}
