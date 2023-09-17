use rspack_core::tree_shaking::symbol::{self, IndirectTopLevelSymbol};
use rspack_core::tree_shaking::visitor::SymbolRef;
use rspack_core::{
  import_statement, ConnectionState, Dependency, DependencyCategory, DependencyCondition,
  DependencyId, DependencyTemplate, DependencyType, ErrorSpan, ExtendedReferencedExport,
  InitFragmentStage, ModuleDependency, ModuleIdentifier, NormalInitFragment, RuntimeGlobals,
  TemplateContext, TemplateReplaceSource,
};
use rspack_core::{ModuleGraph, RuntimeSpec};
use rustc_hash::FxHashSet as HashSet;
use swc_core::ecma::atoms::JsWord;

use super::create_resource_identifier_for_esm_dependency;

#[derive(Debug, Clone)]
pub enum Specifier {
  Namespace(JsWord),
  Default(JsWord),
  Named(JsWord, Option<JsWord>),
}

// HarmonyImportDependency is merged HarmonyImportSideEffectDependency.
#[derive(Debug, Clone)]
pub struct HarmonyImportDependency {
  pub request: JsWord,
  pub id: DependencyId,
  pub span: Option<ErrorSpan>,
  pub specifiers: Vec<Specifier>,
  pub dependency_type: DependencyType,
  pub export_all: bool,
  resource_identifier: String,
}

impl HarmonyImportDependency {
  pub fn new(
    request: JsWord,
    span: Option<ErrorSpan>,
    specifiers: Vec<Specifier>,
    dependency_type: DependencyType,
    export_all: bool,
  ) -> Self {
    let resource_identifier = create_resource_identifier_for_esm_dependency(&request);
    Self {
      id: DependencyId::new(),
      request,
      span,
      specifiers,
      dependency_type,
      export_all,
      resource_identifier,
    }
  }
}

impl DependencyTemplate for HarmonyImportDependency {
  fn apply(
    &self,
    _source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let compilation = &code_generatable_context.compilation;
    let module = &code_generatable_context.module;

    let ref_mgm = compilation
      .module_graph
      .module_graph_module_by_dependency_id(&self.id)
      .expect("should have ref module");
    if !compilation
      .include_module_ids
      .contains(&ref_mgm.module_identifier)
    {
      return;
    }

    if !self.export_all {
      let specifiers = self
        .specifiers
        .iter()
        .filter(|specifier| {
          let is_import = matches!(self.dependency_type, DependencyType::EsmImport);
          if is_import && !ref_mgm.module_type.is_js_like() {
            return true;
          }

          match specifier {
            Specifier::Namespace(_) => true,
            Specifier::Default(local) => {
              if is_import {
                compilation
                  .used_symbol_ref
                  .contains(&SymbolRef::Indirect(IndirectTopLevelSymbol {
                    src: ref_mgm.module_identifier,
                    ty: symbol::IndirectType::ImportDefault(local.clone()),
                    importer: module.identifier(),
                    dep_id: self.id,
                  }))
              } else {
                unreachable!("`export v from ''` is a unrecoverable syntax error")
              }
            }
            Specifier::Named(local, imported) => {
              let symbol = if matches!(self.dependency_type, DependencyType::EsmImport) {
                SymbolRef::Indirect(IndirectTopLevelSymbol {
                  src: ref_mgm.module_identifier,
                  ty: symbol::IndirectType::Import(local.clone(), imported.clone()),
                  importer: module.identifier(),
                  dep_id: self.id,
                })
              } else {
                SymbolRef::Indirect(IndirectTopLevelSymbol {
                  src: module.identifier(),
                  ty: symbol::IndirectType::ReExport(local.clone(), imported.clone()),
                  importer: module.identifier(),
                  dep_id: self.id,
                })
              };

              compilation.used_symbol_ref.contains(&symbol)
            }
          }
        })
        .collect::<Vec<_>>();

      if specifiers.is_empty()
        && compilation
          .side_effects_free_modules
          .contains(&ref_mgm.module_identifier)
      {
        return;
      }
    }

    let content: (String, String) =
      import_statement(code_generatable_context, &self.id, &self.request, false);

    let TemplateContext {
      init_fragments,
      compilation,
      module,
      runtime_requirements,
      ..
    } = code_generatable_context;

    let ref_module = compilation
      .module_graph
      .module_identifier_by_dependency_id(&self.id)
      .expect("should have dependency referenced module");
    let import_var = compilation
      .module_graph
      .get_import_var(&module.identifier(), &self.request);
    if compilation.module_graph.is_async(ref_module) {
      init_fragments.push(Box::new(NormalInitFragment::new(
        content.0,
        InitFragmentStage::StageHarmonyImports,
        None,
      )));
      init_fragments.push(Box::new(NormalInitFragment::new(
        format!(
          "var __webpack_async_dependencies__ = __webpack_handle_async_dependencies__([{import_var}]);\n([{import_var}] = __webpack_async_dependencies__.then ? (await __webpack_async_dependencies__)() : __webpack_async_dependencies__);"
        ),
        InitFragmentStage::StageHarmonyImports,
        None,
      )));
      init_fragments.push(Box::new(NormalInitFragment::new(
        content.1,
        InitFragmentStage::StageAsyncHarmonyImports,
        None,
      )));
    } else {
      init_fragments.push(Box::new(NormalInitFragment::new(
        format!("{}{}", content.0, content.1),
        InitFragmentStage::StageHarmonyImports,
        None,
      )));
    }
    if self.export_all {
      runtime_requirements.insert(RuntimeGlobals::EXPORT_STAR);
      let exports_argument = compilation
        .module_graph
        .module_graph_module_by_identifier(&module.identifier())
        .expect("should have mgm")
        .get_exports_argument();
      init_fragments.push(Box::new(NormalInitFragment::new(
        format!(
          "{}.{}({import_var}, {exports_argument});\n",
          RuntimeGlobals::REQUIRE,
          RuntimeGlobals::EXPORT_STAR,
        ),
        if compilation.module_graph.is_async(ref_module) {
          InitFragmentStage::StageAsyncHarmonyImports
        } else {
          InitFragmentStage::StageHarmonyImports
        },
        None,
      )));
    }
  }
}

impl Dependency for HarmonyImportDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &self.dependency_type
  }
}

impl ModuleDependency for HarmonyImportDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn span(&self) -> Option<&ErrorSpan> {
    self.span.as_ref()
  }

  fn set_request(&mut self, request: String) {
    self.request = request.into();
  }

  fn resource_identifier(&self) -> Option<&str> {
    Some(&self.resource_identifier)
  }

  fn get_referenced_exports(
    &self,
    _module_graph: &ModuleGraph,
    _runtime: Option<&RuntimeSpec>,
  ) -> Vec<ExtendedReferencedExport> {
    vec![]
  }

  // TODO: It's from HarmonyImportSideEffectDependency.
  fn get_condition(&self) -> Option<DependencyCondition> {
    None
    // let id = self.id;
    // Some(DependencyCondition::Fn(Box::new(
    //   move |_, _, module_graph| {
    //     if let Some(module) = module_graph
    //       .parent_module_by_dependency_id(&id)
    //       .and_then(|module_identifier| module_graph.module_by_identifier(&module_identifier))
    //     {
    //       module.get_side_effects_connection_state(module_graph, &mut HashSet::default())
    //     } else {
    //       ConnectionState::Bool(true)
    //     }
    //   },
    // )))
  }

  // It's from HarmonyImportSideEffectDependency.
  fn get_module_evaluation_side_effects_state(
    &self,
    module_graph: &ModuleGraph,
    module_chain: &mut HashSet<ModuleIdentifier>,
  ) -> ConnectionState {
    if let Some(module) = module_graph
      .parent_module_by_dependency_id(&self.id)
      .and_then(|module_identifier| module_graph.module_by_identifier(&module_identifier))
    {
      module.get_side_effects_connection_state(module_graph, module_chain)
    } else {
      ConnectionState::Bool(true)
    }
  }
}
