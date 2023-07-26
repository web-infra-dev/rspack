use rspack_core::{
  export_from_import, get_dependency_used_by_exports_condition,
  tree_shaking::{symbol, visitor::SymbolRef},
  Compilation, Dependency, DependencyCategory, DependencyCondition, DependencyId,
  DependencyTemplate, DependencyType, ErrorSpan, ExportsReferencedType, Module, ModuleDependency,
  ModuleGraph, ModuleGraphModule, ReferencedExport, RuntimeSpec, TemplateContext,
  TemplateReplaceSource, UsedByExports,
};
use rustc_hash::FxHashSet as HashSet;
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
  used_by_exports: UsedByExports,
  referenced_properties_in_destructuring: Option<HashSet<JsWord>>,
  resource_identifier: String,
}

impl HarmonyImportSpecifierDependency {
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    request: JsWord,
    shorthand: bool,
    start: u32,
    end: u32,
    ids: Vec<JsWord>,
    is_call: bool,
    specifier: Specifier,
  ) -> Self {
    let resource_identifier = format!("{}|{}", DependencyCategory::Esm, &request);
    Self {
      id: DependencyId::new(),
      request,
      shorthand,
      start,
      end,
      ids,
      is_call,
      specifier,
      used_by_exports: UsedByExports::default(),
      referenced_properties_in_destructuring: None,
      resource_identifier,
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
        compilation.used_symbol_ref.iter().find(|symbol| {
          matches!(
            symbol,
            SymbolRef::Indirect(indirect) if indirect.src == reference_mgm.module_identifier && indirect.ty == symbol::IndirectType::ImportDefault(local.clone()) && indirect.importer() == module.identifier()
          )
        }).is_some()
      }
      Specifier::Named(local, imported) => {
        compilation.used_symbol_ref.iter().find(|symbol| {
          matches!(
            symbol,
            SymbolRef::Indirect(indirect) if indirect.src == reference_mgm.module_identifier && indirect.ty == symbol::IndirectType::Import(local.clone(), imported.clone()) && indirect.importer() == module.identifier()
          )
        }).is_some()
      }
    }
  }

  pub fn get_referenced_exports_in_destructuring(
    &self,
    ids: Option<&Vec<JsWord>>,
  ) -> ExportsReferencedType {
    if let Some(referenced_properties) = &self.referenced_properties_in_destructuring {
      ExportsReferencedType::Value(
        referenced_properties
          .iter()
          .map(|prop| {
            if let Some(v) = ids {
              let mut value = v.clone();
              value.push(prop.clone());
              ReferencedExport::new(value, false)
            } else {
              ReferencedExport::new(vec![prop.clone()], false)
            }
          })
          .collect(),
      )
    } else {
      if let Some(v) = ids {
        ExportsReferencedType::Value(vec![ReferencedExport::new(v.clone(), true)])
      } else {
        ExportsReferencedType::Object
      }
    }
  }
}

impl Dependency for HarmonyImportSpecifierDependency {
  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::EsmImportSpecifier
  }

  fn resource_identifier(&self) -> Option<&str> {
    Some(&self.resource_identifier)
  }
}

impl ModuleDependency for HarmonyImportSpecifierDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn span(&self) -> Option<&ErrorSpan> {
    None
  }

  fn as_code_generatable_dependency(&self) -> Option<&dyn DependencyTemplate> {
    Some(self)
  }

  fn set_request(&mut self, request: String) {
    self.request = request.into();
  }

  fn get_condition(&self, module_graph: &ModuleGraph) -> DependencyCondition {
    get_dependency_used_by_exports_condition(&self.id, &self.used_by_exports, module_graph)
  }

  fn get_referenced_exports(
    &self,
    _module_graph: &ModuleGraph,
    _runtime: &RuntimeSpec,
  ) -> ExportsReferencedType {
    if self.ids.is_empty() {
      return self.get_referenced_exports_in_destructuring(None);
    }
    // TODO
    return self.get_referenced_exports_in_destructuring(Some(&self.ids));
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
      let value = format!("/* \"{}\" unused */null", self.request);
      if self.shorthand {
        source.insert(self.end, &format!(": {value}"), None);
      } else {
        source.replace(self.start, self.end, &value, None)
      }
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
