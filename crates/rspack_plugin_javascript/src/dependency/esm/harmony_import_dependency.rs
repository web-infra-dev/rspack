use std::sync::Arc;

use rspack_core::tree_shaking::symbol::{self, IndirectTopLevelSymbol};
use rspack_core::tree_shaking::visitor::SymbolRef;
use rspack_core::{
  get_import_var, import_statement, AsContextDependency, AwaitDependenciesInitFragment,
  ConnectionState, Dependency, DependencyCategory, DependencyCondition, DependencyId,
  DependencyTemplate, DependencyType, ErrorSpan, ExtendedReferencedExport, InitFragmentExt,
  InitFragmentKey, InitFragmentStage, ModuleDependency, ModuleIdentifier, NormalInitFragment,
  RuntimeGlobals, TemplateContext, TemplateReplaceSource,
};
use rspack_core::{ModuleGraph, RuntimeSpec};
use rustc_hash::FxHashSet as HashSet;
use swc_core::ecma::atoms::Atom;

use super::create_resource_identifier_for_esm_dependency;

#[derive(Debug, Clone)]
pub enum Specifier {
  Namespace(Atom),
  Default(Atom),
  Named(Atom, Option<Atom>),
}

// HarmonyImportDependency is merged HarmonyImportSideEffectDependency.
#[derive(Debug, Clone)]
pub struct HarmonyImportSideEffectDependency {
  pub request: Atom,
  pub source_order: i32,
  pub id: DependencyId,
  pub span: Option<ErrorSpan>,
  pub source_span: Option<ErrorSpan>,
  pub specifiers: Vec<Specifier>,
  pub dependency_type: DependencyType,
  pub export_all: bool,
  resource_identifier: String,
}

impl HarmonyImportSideEffectDependency {
  pub fn new(
    request: Atom,
    source_order: i32,
    span: Option<ErrorSpan>,
    source_span: Option<ErrorSpan>,
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
      source_span,
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
  let TemplateContext {
    compilation,
    module,
    runtime,
    ..
  } = code_generatable_context;
  // Only available when module factorization is successful.
  let ref_mgm = compilation
    .module_graph
    .module_graph_module_by_dependency_id(module_dependency.id());
  let is_target_active = if compilation.options.is_new_tree_shaking() {
    let connection = compilation
      .module_graph
      .connection_by_dependency(module_dependency.id());
    if let Some(con) = connection {
      Some(con.is_target_active(&compilation.module_graph, *runtime))
    } else {
      Some(true)
    }
  } else if let Some(ref_mgm) = ref_mgm {
    Some(
      compilation
        .include_module_ids
        .contains(&ref_mgm.module_identifier),
    )
  } else {
    // This represents if module does not exist.
    None
  };
  // Bailout only if the module does exist and not active.
  if is_target_active.is_some_and(|x| !x) {
    return;
  }
  if let Some(ref_mgm) = ref_mgm
    && module_dependency.is_export_all() == Some(false)
  {
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
    .module_identifier_by_dependency_id(module_dependency.id());
  let import_var = get_import_var(&compilation.module_graph, *module_dependency.id());
  //
  // https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/dependencies/HarmonyImportDependency.js#L282-L285
  let module_key = ref_module
    .map(|i| i.as_str())
    .unwrap_or(module_dependency.request());
  let key = format!("harmony import {}", module_key);
  let is_async_module = matches!(ref_module, Some(ref_module) if compilation.module_graph.is_async(ref_module) == Some(true));
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

  let is_new_tree_shaking = compilation.options.is_new_tree_shaking();
  if module_dependency.is_export_all() == Some(true) && !is_new_tree_shaking {
    runtime_requirements.insert(RuntimeGlobals::EXPORT_STAR);
    runtime_requirements.insert(RuntimeGlobals::REQUIRE);
    let exports_argument = compilation
      .module_graph
      .module_by_identifier(&module.identifier())
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
    "HarmonyImportSideEffectDependency"
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

  fn resource_identifier(&self) -> Option<&str> {
    Some(&self.resource_identifier)
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

  fn source_span(&self) -> Option<ErrorSpan> {
    self.source_span
  }

  fn set_request(&mut self, request: String) {
    self.request = request.into();
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
    Some(DependencyCondition::Fn(Arc::new(
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

impl AsContextDependency for HarmonyImportSideEffectDependency {}
