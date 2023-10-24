use rspack_core::tree_shaking::symbol::{self, IndirectTopLevelSymbol};
use rspack_core::tree_shaking::visitor::SymbolRef;
use rspack_core::{
  import_statement, AwaitDependenciesInitFragment, ConnectionState, Dependency, DependencyCategory,
  DependencyCondition, DependencyId, DependencyTemplate, DependencyType, ErrorSpan,
  ExtendedReferencedExport, InitFragmentExt, InitFragmentKey, InitFragmentStage, ModuleDependency,
  ModuleIdentifier, NormalInitFragment, RuntimeGlobals, TemplateContext, TemplateReplaceSource,
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
pub struct HarmonyImportSideEffectDependency {
  pub request: JsWord,
  pub source_order: i32,
  pub id: DependencyId,
  pub span: Option<ErrorSpan>,
  pub specifiers: Vec<Specifier>,
  pub dependency_type: DependencyType,
  pub export_all: bool,
  resource_identifier: String,
}

impl HarmonyImportSideEffectDependency {
  pub fn new(
    request: JsWord,
    source_order: i32,
    span: Option<ErrorSpan>,
    specifiers: Vec<Specifier>,
    dependency_type: DependencyType,
    export_all: bool,
  ) -> Self {
    let resource_identifier = create_resource_identifier_for_esm_dependency(&request);
    Self {
      id: DependencyId::new(),
      source_order,
      request,
      span,
      specifiers,
      dependency_type,
      export_all,
      resource_identifier,
    }
  }
}

pub fn harmony_import_dependency_apply<T: ModuleDependency>(
  module_dependency: &T,
  source_order: i32,
  code_generatable_context: &mut TemplateContext,
  specifiers: &[Specifier],
) {
  let compilation = &code_generatable_context.compilation;
  let module = &code_generatable_context.module;
  let ref_mgm = compilation
    .module_graph
    .module_graph_module_by_dependency_id(module_dependency.id())
    .expect("should have ref module");
  let is_target_active = if compilation.options.is_new_tree_shaking() {
    let connection = compilation
      .module_graph
      .connection_by_dependency(module_dependency.id());
    if let Some(con) = connection {
      // TODO: runtime opt
      // dbg!(
      //   &con,
      //   &ret,
      //   &compilation
      //     .module_graph
      //     .dependency_by_id(&con.dependency_id)
      //     .and_then(|item| item.as_module_dependency())
      //     .map(|item| item.dependency_debug_name())
      // );
      con.is_target_active(&compilation.module_graph, None)
    } else {
      true
    }
  } else {
    compilation
      .include_module_ids
      .contains(&ref_mgm.module_identifier)
  };
  if !is_target_active {
    return;
  }
  if module_dependency.is_export_all() == Some(false) {
    let specifiers = specifiers
      .iter()
      .filter(|specifier| {
        let is_import = matches!(
          module_dependency.dependency_type(),
          DependencyType::EsmImport(_)
        );
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
                  dep_id: *module_dependency.id(),
                }))
            } else {
              unreachable!("`export v from ''` is a unrecoverable syntax error")
            }
          }
          Specifier::Named(local, imported) => {
            let symbol = if matches!(
              module_dependency.dependency_type(),
              DependencyType::EsmImport(_)
            ) {
              SymbolRef::Indirect(IndirectTopLevelSymbol {
                src: ref_mgm.module_identifier,
                ty: symbol::IndirectType::Import(local.clone(), imported.clone()),
                importer: module.identifier(),
                dep_id: *module_dependency.id(),
              })
            } else {
              SymbolRef::Indirect(IndirectTopLevelSymbol {
                src: module.identifier(),
                ty: symbol::IndirectType::ReExport(local.clone(), imported.clone()),
                importer: module.identifier(),
                dep_id: *module_dependency.id(),
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
  let content: (String, String) = import_statement(
    code_generatable_context,
    module_dependency.id(),
    module_dependency.request(),
    false,
  );
  let TemplateContext {
    init_fragments,
    compilation,
    module,
    runtime_requirements,
    ..
  } = code_generatable_context;
  let ref_module = compilation
    .module_graph
    .module_identifier_by_dependency_id(module_dependency.id())
    .expect("should have dependency referenced module");
  let import_var = compilation
    .module_graph
    .get_import_var(&module.identifier(), module_dependency.request());
  let key = module_dependency.request();
  let is_async_module = matches!(compilation.module_graph.is_async(ref_module), Some(true));
  if is_async_module {
    init_fragments.push(Box::new(NormalInitFragment::new(
      content.0,
      InitFragmentStage::StageHarmonyImports,
      source_order,
      InitFragmentKey::HarmonyImport(key.to_string()),
      None,
    )));
    init_fragments.push(AwaitDependenciesInitFragment::new_single(import_var.to_string()).boxed());
    init_fragments.push(Box::new(NormalInitFragment::new(
      content.1,
      InitFragmentStage::StageAsyncHarmonyImports,
      source_order,
      InitFragmentKey::HarmonyImport(format!("{} compat", key)),
      None,
    )));
  } else {
    init_fragments.push(Box::new(NormalInitFragment::new(
      format!("{}{}", content.0, content.1),
      InitFragmentStage::StageHarmonyImports,
      source_order,
      InitFragmentKey::HarmonyImport(key.to_string()),
      None,
    )));
  }

  if module_dependency.is_export_all() == Some(true) {
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
      if is_async_module {
        InitFragmentStage::StageAsyncHarmonyImports
      } else {
        InitFragmentStage::StageHarmonyImports
      },
      source_order,
      InitFragmentKey::HarmonyExportStar(key.to_string()),
      None,
    )));
  }
}

impl Dependency for HarmonyImportSideEffectDependency {
  fn dependency_debug_name(&self) -> &'static str {
    "HarmonyImportDependency"
  }

  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn span(&self) -> Option<ErrorSpan> {
    self.span
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &self.dependency_type
  }

  fn get_module_evaluation_side_effects_state(
    &self,
    module_graph: &ModuleGraph,
    module_chain: &mut HashSet<ModuleIdentifier>,
  ) -> ConnectionState {
    if let Some(module) = module_graph
      .module_identifier_by_dependency_id(&self.id)
      .and_then(|module_identifier| module_graph.module_by_identifier(module_identifier))
    {
      module.get_side_effects_connection_state(module_graph, module_chain)
    } else {
      ConnectionState::Bool(true)
    }
  }
}

impl ModuleDependency for HarmonyImportSideEffectDependency {
  fn is_export_all(&self) -> Option<bool> {
    Some(self.export_all)
  }

  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
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
    Some(DependencyCondition::Fn(Box::new(
      move |con, _, module_graph: &ModuleGraph| {
        let id = con.module_identifier;
        if let Some(module) = module_graph.module_by_identifier(&id) {
          module.get_side_effects_connection_state(module_graph, &mut HashSet::default())
        } else {
          ConnectionState::Bool(true)
        }
      },
    )))
  }

  // It's from HarmonyImportSideEffectDependency.
}

impl DependencyTemplate for HarmonyImportSideEffectDependency {
  fn apply(
    &self,
    _source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    harmony_import_dependency_apply(
      self,
      self.source_order,
      code_generatable_context,
      &self.specifiers,
    );
  }
}
