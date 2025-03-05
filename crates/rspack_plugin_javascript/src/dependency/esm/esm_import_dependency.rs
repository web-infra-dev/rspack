use std::sync::Arc;

use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsPreset, Skip},
};
use rspack_collections::IdentifierSet;
use rspack_core::DependencyConditionFn;
use rspack_core::DependencyLocation;
use rspack_core::DependencyRange;
use rspack_core::SharedSourceMap;
use rspack_core::{
  filter_runtime, import_statement, merge_runtime, AsContextDependency,
  AwaitDependenciesInitFragment, BuildMetaDefaultObject, ConditionalInitFragment, ConnectionState,
  Dependency, DependencyCategory, DependencyCondition, DependencyId, DependencyTemplate,
  DependencyType, ErrorSpan, ExportInfoProvided, ExportsType, ExtendedReferencedExport,
  ImportAttributes, InitFragmentExt, InitFragmentKey, InitFragmentStage, ModuleDependency,
  ProvidedExports, RuntimeCondition, TemplateContext, TemplateReplaceSource,
};
use rspack_core::{Compilation, FactorizeInfo};
use rspack_core::{ModuleGraph, RuntimeSpec};
use rspack_error::miette::{MietteDiagnostic, Severity};
use rspack_error::DiagnosticExt;
use rspack_error::{Diagnostic, TraceableError};
use swc_core::ecma::atoms::Atom;

use super::create_resource_identifier_for_esm_dependency;

// TODO: find a better way to implement this for performance
// Align with https://github.com/webpack/webpack/blob/51f0f0aeac072f989f8d40247f6c23a1995c5c37/lib/dependencies/HarmonyImportDependency.js#L361-L365
// This map is used to save the runtime conditions of modules and used by ESMAcceptDependency in hot module replacement.
// It can not be saved in TemplateContext because only dependencies of rebuild modules will be templated again.
pub mod import_emitted_runtime {
  use once_cell::sync::OnceCell;
  use rspack_collections::{IdentifierDashMap, IdentifierMap};
  use rspack_core::{ModuleIdentifier, RuntimeCondition};

  static IMPORT_EMITTED_MAP: OnceCell<IdentifierDashMap<IdentifierMap<RuntimeCondition>>> =
    OnceCell::new();

  pub fn init_map() {
    IMPORT_EMITTED_MAP.get_or_init(Default::default);
  }

  pub fn get_map() -> Option<&'static IdentifierDashMap<IdentifierMap<RuntimeCondition>>> {
    IMPORT_EMITTED_MAP.get()
  }

  pub fn get_runtime(
    module: &ModuleIdentifier,
    referenced_module: &ModuleIdentifier,
  ) -> RuntimeCondition {
    let map = get_map().expect("must call import_emitted_runtime::init_map() before");
    let Some(condition_map) = map.get(module) else {
      return RuntimeCondition::Boolean(false);
    };
    match condition_map.get(referenced_module) {
      Some(r) => r.to_owned(),
      None => RuntimeCondition::Boolean(false),
    }
  }
}

// ESMImportDependency is merged ESMImportSideEffectDependency.
#[cacheable]
#[derive(Debug, Clone)]
pub struct ESMImportSideEffectDependency {
  #[cacheable(with=AsPreset)]
  pub request: Atom,
  pub source_order: i32,
  pub id: DependencyId,
  pub range: DependencyRange,
  pub range_src: DependencyRange,
  pub dependency_type: DependencyType,
  pub export_all: bool,
  attributes: Option<ImportAttributes>,
  resource_identifier: String,
  #[cacheable(with=Skip)]
  source_map: Option<SharedSourceMap>,
  factorize_info: FactorizeInfo,
}

impl ESMImportSideEffectDependency {
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    request: Atom,
    source_order: i32,
    range: DependencyRange,
    range_src: DependencyRange,
    dependency_type: DependencyType,
    export_all: bool,
    attributes: Option<ImportAttributes>,
    source_map: Option<SharedSourceMap>,
  ) -> Self {
    let resource_identifier =
      create_resource_identifier_for_esm_dependency(&request, attributes.as_ref());
    Self {
      id: DependencyId::new(),
      source_order,
      request,
      range,
      range_src,
      dependency_type,
      export_all,
      attributes,
      resource_identifier,
      source_map,
      factorize_info: Default::default(),
    }
  }
}

pub fn esm_import_dependency_apply<T: ModuleDependency>(
  module_dependency: &T,
  source_order: i32,
  code_generatable_context: &mut TemplateContext,
) {
  let TemplateContext {
    compilation,
    module,
    runtime,
    runtime_requirements,
    ..
  } = code_generatable_context;
  // Only available when module factorization is successful.
  let module_graph = compilation.get_module_graph();
  let connection = module_graph.connection_by_dependency_id(module_dependency.id());
  let is_target_active = if let Some(con) = connection {
    Some(con.is_target_active(&module_graph, *runtime))
  } else {
    Some(true)
  };
  // Bailout only if the module does exist and not active.
  if is_target_active.is_some_and(|x| !x) {
    return;
  }

  let runtime_condition = if module_dependency.weak() {
    RuntimeCondition::Boolean(false)
  } else if let Some(connection) = module_graph.connection_by_dependency_id(module_dependency.id())
  {
    filter_runtime(*runtime, |r| connection.is_target_active(&module_graph, r))
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
    ..
  } = code_generatable_context;
  let ref_module = module_graph.module_identifier_by_dependency_id(module_dependency.id());
  let import_var = compilation.get_import_var(module_dependency.id());

  // https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/dependencies/HarmonyImportDependency.js#L282-L285
  let module_key = ref_module
    .map(|i| i.as_str())
    .unwrap_or(module_dependency.request());
  let key = format!("ESM import {}", module_key);

  // The import emitted map is consumed by ESMAcceptDependency which enabled by HotModuleReplacementPlugin
  if let Some(import_emitted_map) = import_emitted_runtime::get_map() {
    if let Some(ref_module) = ref_module {
      let mut emitted_modules = import_emitted_map.entry(module.identifier()).or_default();

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

  let is_async_module =
    matches!(ref_module, Some(ref_module) if ModuleGraph::is_async(compilation, ref_module));
  if is_async_module {
    init_fragments.push(Box::new(ConditionalInitFragment::new(
      content.0,
      InitFragmentStage::StageESMImports,
      source_order,
      InitFragmentKey::ESMImport(key.to_string()),
      None,
      runtime_condition.clone(),
    )));
    init_fragments.push(AwaitDependenciesInitFragment::new_single(import_var.to_string()).boxed());
    init_fragments.push(Box::new(ConditionalInitFragment::new(
      content.1,
      InitFragmentStage::StageAsyncESMImports,
      source_order,
      InitFragmentKey::ESMImport(format!("{} compat", key)),
      None,
      runtime_condition,
    )));
  } else {
    init_fragments.push(Box::new(ConditionalInitFragment::new(
      format!("{}{}", content.0, content.1),
      InitFragmentStage::StageESMImports,
      source_order,
      InitFragmentKey::ESMImport(key.to_string()),
      None,
      runtime_condition,
    )));
  }
}

pub fn esm_import_dependency_get_linking_error<T: ModuleDependency>(
  module_dependency: &T,
  ids: &[Atom],
  module_graph: &ModuleGraph,
  additional_msg: String,
  should_error: bool,
) -> Option<Diagnostic> {
  let imported_module = module_graph.get_module_by_dependency_id(module_dependency.id())?;
  if !imported_module.diagnostics().is_empty() {
    return None;
  }
  let parent_module_identifier = module_graph
    .get_parent_module(module_dependency.id())
    .expect("should have parent module for dependency");
  let parent_module = module_graph
    .module_by_identifier(parent_module_identifier)
    .expect("should have module");
  let exports_type =
    imported_module.get_exports_type(module_graph, parent_module.build_meta().strict_esm_module);
  let create_error = |message: String| {
    let (severity, title) = if should_error {
      (Severity::Error, "ESModulesLinkingError")
    } else {
      (Severity::Warning, "ESModulesLinkingWarning")
    };
    let mut diagnostic = if let Some(span) = module_dependency.range()
      && let Some(source) = parent_module.source()
    {
      Diagnostic::from(
        TraceableError::from_file(
          source.source().into_owned(),
          span.start as usize,
          span.end as usize,
          title.to_string(),
          message,
        )
        .with_severity(severity)
        .boxed(),
      )
      .with_hide_stack(Some(true))
    } else {
      Diagnostic::from(
        MietteDiagnostic::new(message)
          .with_code(title)
          .with_severity(severity)
          .boxed(),
      )
      .with_hide_stack(Some(true))
    };
    diagnostic = diagnostic.with_module_identifier(Some(*parent_module_identifier));
    diagnostic
  };
  if matches!(
    exports_type,
    ExportsType::Namespace | ExportsType::DefaultWithNamed
  ) {
    if ids.is_empty() {
      return None;
    }
    let imported_module_identifier = imported_module.identifier();
    if (!matches!(exports_type, ExportsType::DefaultWithNamed) || ids[0] != "default")
      && matches!(
        module_graph.is_export_provided(&imported_module_identifier, ids),
        Some(false)
      )
    {
      let mut pos = 0;
      let mut maybe_exports_info = Some(module_graph.get_exports_info(&imported_module_identifier));
      while pos < ids.len()
        && let Some(exports_info) = maybe_exports_info
      {
        let id = &ids[pos];
        pos += 1;
        let export_info = exports_info.get_read_only_export_info(module_graph, id);
        if matches!(
          export_info.provided(module_graph),
          Some(ExportInfoProvided::False)
        ) {
          let provided_exports = exports_info.get_provided_exports(module_graph);
          let more_info = if let ProvidedExports::Vec(exports) = &provided_exports {
            if exports.is_empty() {
              " (module has no exports)".to_string()
            } else {
              format!(
                " (possible exports: {})",
                exports
                  .iter()
                  .map(|e| e.as_str())
                  .collect::<Vec<_>>()
                  .join(", ")
              )
            }
          } else {
            " (possible exports unknown)".to_string()
          };
          let msg = format!(
            "export {} {} was not found in '{}'{more_info}",
            ids
              .iter()
              .take(pos)
              .map(|id| format!("'{id}'"))
              .collect::<Vec<_>>()
              .join("."),
            additional_msg,
            module_dependency.user_request(),
          );
          return Some(create_error(msg));
        }
        maybe_exports_info = export_info.get_nested_exports_info(module_graph);
      }
      let msg = format!(
        "export {} {} was not found in '{}'",
        ids
          .iter()
          .map(|id| format!("'{id}'"))
          .collect::<Vec<_>>()
          .join("."),
        additional_msg,
        module_dependency.user_request()
      );
      return Some(create_error(msg));
    }
  }
  match exports_type {
    ExportsType::DefaultOnly => {
      if !ids.is_empty() && ids[0] != "default" {
        let msg = format!(
          "Can't import the named export {} {} from default-exporting module (only default export is available)",
          ids
            .iter()
            .map(|id| format!("'{id}'"))
            .collect::<Vec<_>>()
            .join("."),
          additional_msg,
        );
        return Some(create_error(msg));
      }
    }
    ExportsType::DefaultWithNamed => {
      if !ids.is_empty()
        && ids[0] != "default"
        && matches!(
          imported_module.build_meta().default_object,
          BuildMetaDefaultObject::RedirectWarn { ignore: false }
        )
      {
        let msg = format!(
          "Should not import the named export {} {} from default-exporting module (only default export is available soon)",
          ids
            .iter()
            .map(|id| format!("'{id}'"))
            .collect::<Vec<_>>()
            .join("."),
          additional_msg,
        );
        return Some(create_error(msg));
      }
    }
    _ => {}
  }
  None
}

#[cacheable_dyn]
impl Dependency for ESMImportSideEffectDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn loc(&self) -> Option<DependencyLocation> {
    self.range.to_loc(self.source_map.as_ref())
  }

  fn range(&self) -> Option<&DependencyRange> {
    Some(&self.range)
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

  fn get_attributes(&self) -> Option<&ImportAttributes> {
    self.attributes.as_ref()
  }

  fn get_module_evaluation_side_effects_state(
    &self,
    module_graph: &ModuleGraph,
    module_chain: &mut IdentifierSet,
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

  fn get_referenced_exports(
    &self,
    _module_graph: &ModuleGraph,
    _runtime: Option<&RuntimeSpec>,
  ) -> Vec<ExtendedReferencedExport> {
    vec![]
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

struct ESMImportSideEffectDependencyCondition;

impl DependencyConditionFn for ESMImportSideEffectDependencyCondition {
  fn get_connection_state(
    &self,
    conn: &rspack_core::ModuleGraphConnection,
    _runtime: Option<&RuntimeSpec>,
    module_graph: &ModuleGraph,
  ) -> ConnectionState {
    let id = *conn.module_identifier();
    if let Some(module) = module_graph.module_by_identifier(&id) {
      module.get_side_effects_connection_state(module_graph, &mut IdentifierSet::default())
    } else {
      ConnectionState::Bool(true)
    }
  }
}

#[cacheable_dyn]
impl ModuleDependency for ESMImportSideEffectDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn source_span(&self) -> Option<ErrorSpan> {
    Some(ErrorSpan::new(self.range_src.start, self.range_src.end))
  }

  fn set_request(&mut self, request: String) {
    self.request = request.into();
  }

  // TODO: It's from ESMImportSideEffectDependency.
  fn get_condition(&self) -> Option<DependencyCondition> {
    Some(DependencyCondition::Fn(Arc::new(
      ESMImportSideEffectDependencyCondition,
    )))
  }

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}

#[cacheable_dyn]
impl DependencyTemplate for ESMImportSideEffectDependency {
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
    let module_graph = compilation.get_module_graph();
    if let Some(scope) = concatenation_scope {
      let module = module_graph.get_module_by_dependency_id(&self.id);
      if module.is_some_and(|m| scope.is_module_in_scope(&m.identifier())) {
        return;
      }
    }
    esm_import_dependency_apply(self, self.source_order, code_generatable_context);
  }

  fn dependency_id(&self) -> Option<DependencyId> {
    Some(self.id)
  }

  fn update_hash(
    &self,
    _hasher: &mut dyn std::hash::Hasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
  }
}

impl AsContextDependency for ESMImportSideEffectDependency {}
