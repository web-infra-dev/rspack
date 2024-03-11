use std::sync::Arc;

use dashmap::DashMap;
use once_cell::sync::Lazy;
use rspack_core::tree_shaking::symbol::{self, IndirectTopLevelSymbol};
use rspack_core::tree_shaking::visitor::SymbolRef;
use rspack_core::{
  filter_runtime, get_import_var, import_statement, merge_runtime, AsContextDependency,
  AwaitDependenciesInitFragment, ConditionalInitFragment, ConnectionState, Dependency,
  DependencyCategory, DependencyCondition, DependencyId, DependencyTemplate, DependencyType,
  ErrorSpan, ExtendedReferencedExport, InitFragmentExt, InitFragmentKey, InitFragmentStage,
  ModuleDependency, ModuleIdentifier, NormalInitFragment, RuntimeCondition, RuntimeGlobals,
  TemplateContext, TemplateReplaceSource,
};
use rspack_core::{ModuleGraph, RuntimeSpec};
use rustc_hash::{FxHashMap, FxHashSet as HashSet};
use swc_core::ecma::atoms::Atom;

use super::create_resource_identifier_for_esm_dependency;

// TODO: find a better way to implement this for performance
// Align with https://github.com/webpack/webpack/blob/51f0f0aeac072f989f8d40247f6c23a1995c5c37/lib/dependencies/HarmonyImportDependency.js#L361-L365
// This map is used to save the runtime conditions of modules and used by HarmonyAcceptDependency in hot module replacement.
// It can not be saved in TemplateContext because only dependencies of rebuild modules will be templated again.
static IMPORT_EMITTED_MAP: Lazy<
  DashMap<ModuleIdentifier, FxHashMap<ModuleIdentifier, RuntimeCondition>>,
> = Lazy::new(Default::default);

pub fn get_import_emitted_runtime(
  module: &ModuleIdentifier,
  referenced_module: &ModuleIdentifier,
) -> RuntimeCondition {
  let Some(condition_map) = IMPORT_EMITTED_MAP.get(module) else {
    return RuntimeCondition::Boolean(false);
  };
  match condition_map.get(referenced_module) {
    Some(r) => r.to_owned(),
    None => RuntimeCondition::Boolean(false),
  }
}

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
    runtime_requirements,
    ..
  } = code_generatable_context;
  // Only available when module factorization is successful.
  let ref_mgm = compilation
    .get_module_graph()
    .module_graph_module_by_dependency_id(module_dependency.id());
  let is_target_active = if compilation.options.is_new_tree_shaking() {
    let connection = compilation
      .get_module_graph()
      .connection_by_dependency(module_dependency.id());
    if let Some(con) = connection {
      Some(con.is_target_active(&compilation.get_module_graph(), *runtime))
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

  let runtime_condition = if let Some(connection) = compilation
    .get_module_graph()
    .connection_by_dependency(module_dependency.id())
  {
    filter_runtime(*runtime, |r| {
      connection.is_target_active(&compilation.get_module_graph(), r)
    })
  } else {
    RuntimeCondition::Boolean(true)
  };

  let content: (String, String) = import_statement(
    *module,
    compilation,
    runtime_requirements,
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
    .get_module_graph()
    .module_identifier_by_dependency_id(module_dependency.id());
  let import_var = get_import_var(&compilation.get_module_graph(), *module_dependency.id());
  //
  // https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/dependencies/HarmonyImportDependency.js#L282-L285
  let module_key = ref_module
    .map(|i| i.as_str())
    .unwrap_or(module_dependency.request());
  let key = format!("harmony import {}", module_key);

  // NOTE: different with webpack
  // The import emitted map is consumed by HarmonyAcceptDependency which enabled by `dev_server.hot`
  if compilation.options.dev_server.hot {
    if let Some(ref_module) = ref_module {
      let mut emitted_modules = IMPORT_EMITTED_MAP.entry(module.identifier()).or_default();

      let old_runtime_condition = match emitted_modules.get(ref_module) {
        Some(v) => v.to_owned(),
        None => RuntimeCondition::Boolean(false),
      };

      let mut merged_runtime_condition = runtime_condition.clone();
      if !matches!(old_runtime_condition, RuntimeCondition::Boolean(false))
        && !matches!(merged_runtime_condition, RuntimeCondition::Boolean(true))
      {
        if matches!(merged_runtime_condition, RuntimeCondition::Boolean(false))
          || matches!(old_runtime_condition, RuntimeCondition::Boolean(true))
        {
          merged_runtime_condition = old_runtime_condition;
        } else {
          merged_runtime_condition = RuntimeCondition::Spec(merge_runtime(
            old_runtime_condition.as_spec().expect("should be spec"),
            merged_runtime_condition.as_spec().expect("should be spec"),
          ));
        }
      }
      emitted_modules.insert(*ref_module, merged_runtime_condition);
    }
  }

  let is_async_module = matches!(ref_module, Some(ref_module) if compilation.get_module_graph().is_async(ref_module) == Some(true));
  if is_async_module {
    init_fragments.push(Box::new(ConditionalInitFragment::new(
      content.0,
      InitFragmentStage::StageHarmonyImports,
      source_order,
      InitFragmentKey::HarmonyImport(key.to_string()),
      None,
      runtime_condition.clone(),
    )));
    init_fragments.push(AwaitDependenciesInitFragment::new_single(import_var.to_string()).boxed());
    init_fragments.push(Box::new(ConditionalInitFragment::new(
      content.1,
      InitFragmentStage::StageAsyncHarmonyImports,
      source_order,
      InitFragmentKey::HarmonyImport(format!("{} compat", key)),
      None,
      runtime_condition,
    )));
  } else {
    init_fragments.push(Box::new(ConditionalInitFragment::new(
      format!("{}{}", content.0, content.1),
      InitFragmentStage::StageHarmonyImports,
      source_order,
      InitFragmentKey::HarmonyImport(key.to_string()),
      None,
      runtime_condition,
    )));
  }

  let is_new_tree_shaking = compilation.options.is_new_tree_shaking();
  if module_dependency.is_export_all() == Some(true) && !is_new_tree_shaking {
    runtime_requirements.insert(RuntimeGlobals::EXPORT_STAR);
    runtime_requirements.insert(RuntimeGlobals::REQUIRE);
    let exports_argument = compilation
      .get_module_graph()
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

  fn source_order(&self) -> Option<i32> {
    Some(self.source_order)
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
    let TemplateContext {
      compilation,
      concatenation_scope,
      ..
    } = code_generatable_context;
    if let Some(scope) = concatenation_scope {
      let module = compilation
        .get_module_graph()
        .get_module(&self.id)
        .expect("should have module");
      if scope.is_module_in_scope(&module.identifier()) {
        return;
      }
    }
    harmony_import_dependency_apply(
      self,
      self.source_order,
      code_generatable_context,
      &self.specifiers,
    );
  }

  fn dependency_id(&self) -> Option<DependencyId> {
    Some(self.id)
  }
}

impl AsContextDependency for HarmonyImportSideEffectDependency {}
