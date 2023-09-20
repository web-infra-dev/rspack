use rspack_core::{
  create_exports_object_referenced, create_no_exports_referenced, export_from_import,
  get_dependency_used_by_exports_condition, get_exports_type,
  tree_shaking::symbol::DEFAULT_JS_WORD, Compilation, ConnectionState, Dependency,
  DependencyCategory, DependencyCondition, DependencyId, DependencyTemplate, DependencyType,
  ErrorSpan, ExportsType, ExtendedReferencedExport, ModuleDependency, ModuleGraph,
  ModuleGraphModule, ModuleIdentifier, ReferencedExport, RuntimeSpec, TemplateContext,
  TemplateReplaceSource, UsedByExports,
};
use rustc_hash::FxHashSet as HashSet;
use swc_core::ecma::atoms::JsWord;

use super::{create_resource_identifier_for_esm_dependency, Specifier};

#[derive(Debug, Clone)]
pub struct HarmonyImportSpecifierDependency {
  id: DependencyId,
  request: JsWord,
  shorthand: bool,
  start: u32,
  end: u32,
  ids: Vec<JsWord>,
  call: bool,
  direct_import: bool,
  specifier: Specifier,
  used_by_exports: UsedByExports,
  namespace_object_as_context: bool,
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
    call: bool,
    direct_import: bool,
    specifier: Specifier,
    referenced_properties_in_destructuring: Option<HashSet<JsWord>>,
  ) -> Self {
    let resource_identifier = create_resource_identifier_for_esm_dependency(&request);
    Self {
      id: DependencyId::new(),
      request,
      shorthand,
      start,
      end,
      ids,
      call,
      direct_import,
      specifier,
      used_by_exports: UsedByExports::default(),
      namespace_object_as_context: false,
      referenced_properties_in_destructuring,
      resource_identifier,
    }
  }

  // TODO move export_info
  pub fn check_used(&self, reference_mgm: &ModuleGraphModule, compilation: &Compilation) -> bool {
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
      Specifier::Default(_) => compilation
        .module_graph
        .get_exports_info(&reference_mgm.module_identifier)
        .old_get_used_exports()
        .contains(&DEFAULT_JS_WORD),
      Specifier::Named(local, imported) => compilation
        .module_graph
        .get_exports_info(&reference_mgm.module_identifier)
        .old_get_used_exports()
        .contains(imported.as_ref().unwrap_or(local)),
    }
  }

  pub fn get_referenced_exports_in_destructuring(
    &self,
    ids: Option<&Vec<JsWord>>,
  ) -> Vec<ExtendedReferencedExport> {
    if let Some(referenced_properties) = &self.referenced_properties_in_destructuring {
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
        .map(ExtendedReferencedExport::Export)
        .collect::<Vec<_>>()
    } else if let Some(v) = ids {
      vec![ReferencedExport::new(v.clone(), true).into()]
    } else {
      create_exports_object_referenced()
    }
  }
}

impl DependencyTemplate for HarmonyImportSpecifierDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let TemplateContext { compilation, .. } = code_generatable_context;

    let reference_mgm = compilation
      .module_graph
      .module_graph_module_by_dependency_id(&self.id)
      .expect("should have ref module");

    let used = self.check_used(reference_mgm, compilation);

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
      self.call,
      !self.direct_import,
    );
    if self.shorthand {
      source.insert(self.end, format!(": {export_expr}").as_str(), None);
    } else {
      source.replace(self.start, self.end, export_expr.as_str(), None)
    }
  }
}

impl Dependency for HarmonyImportSpecifierDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::EsmImportSpecifier
  }
}

impl ModuleDependency for HarmonyImportSpecifierDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn span(&self) -> Option<&ErrorSpan> {
    None
  }

  fn set_request(&mut self, request: String) {
    self.request = request.into();
  }

  fn resource_identifier(&self) -> Option<&str> {
    Some(&self.resource_identifier)
  }

  fn get_condition(&self) -> Option<DependencyCondition> {
    get_dependency_used_by_exports_condition(self.id, &self.used_by_exports)
  }

  fn get_module_evaluation_side_effects_state(
    &self,
    _module_graph: &ModuleGraph,
    _module_chain: &mut HashSet<ModuleIdentifier>,
  ) -> ConnectionState {
    ConnectionState::Bool(false)
  }

  fn get_referenced_exports(
    &self,
    module_graph: &ModuleGraph,
    _runtime: Option<&RuntimeSpec>,
  ) -> Vec<ExtendedReferencedExport> {
    // namespace import
    if self.ids.is_empty() {
      return self.get_referenced_exports_in_destructuring(None);
    }

    let mut ids = vec![];
    let mut namespace_object_as_context = self.namespace_object_as_context;
    if let Some(id) = self.ids.get(0) && id == "default" {
      let parent_module = module_graph.parent_module_by_dependency_id(&self.id).expect("should have parent module");
      let exports_type = get_exports_type(module_graph, &self.id, &parent_module);
      match exports_type {
        ExportsType::DefaultOnly | ExportsType::DefaultWithNamed => {
          if self.ids.len() == 1 {
            return self.get_referenced_exports_in_destructuring(None);
          }
          ids = self.ids.iter().skip(1).collect::<Vec<_>>();
          namespace_object_as_context = true;
        }
        ExportsType::Dynamic => {
          return create_no_exports_referenced();
        }
        _ => {}
      }
    }

    if self.call && !self.direct_import && (namespace_object_as_context || ids.len() > 1) {
      if ids.len() == 1 {
        return create_exports_object_referenced();
      }
      // remove last one
      ids.shrink_to(ids.len() - 1);
    }

    self.get_referenced_exports_in_destructuring(Some(&self.ids))
  }
}
