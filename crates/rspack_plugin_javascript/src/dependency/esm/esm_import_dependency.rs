use rspack_cacheable::{cacheable, cacheable_dyn, with::AsPreset};
use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::{
  AsContextDependency, AwaitDependenciesInitFragment, BuildMetaDefaultObject, ChunkGraph,
  ConditionalInitFragment, ConnectionState, Dependency, DependencyCategory,
  DependencyCodeGeneration, DependencyCondition, DependencyConditionFn, DependencyId,
  DependencyLocation, DependencyRange, DependencyTemplate, DependencyTemplateType, DependencyType,
  ExportProvided, ExportsType, ExtendedReferencedExport, FactorizeInfo, ForwardId,
  ImportAttributes, ImportPhase, InitFragmentExt, InitFragmentKey, InitFragmentStage, LazyUntil,
  ModuleDependency, ModuleGraph, ModuleGraphCacheArtifact, ModuleIdentifier,
  PrefetchExportsInfoMode, ProvidedExports, ResourceIdentifier, RuntimeCondition, RuntimeSpec,
  SharedSourceMap, SourceType, TemplateContext, TemplateReplaceSource, TypeReexportPresenceMode,
  filter_runtime,
};
use rspack_error::{Diagnostic, Error, Severity};
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
  #[cfg(allocative)]
  use rspack_util::allocative;

  #[cfg_attr(allocative, allocative::root)]
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
  request: Atom,
  source_order: i32,
  id: DependencyId,
  range: DependencyRange,
  dependency_type: DependencyType,
  phase: ImportPhase,
  attributes: Option<ImportAttributes>,
  resource_identifier: ResourceIdentifier,
  loc: Option<DependencyLocation>,
  factorize_info: FactorizeInfo,
  lazy_make: bool,
  star_export: bool,
}

impl ESMImportSideEffectDependency {
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    request: Atom,
    source_order: i32,
    range: DependencyRange,
    dependency_type: DependencyType,
    phase: ImportPhase,
    attributes: Option<ImportAttributes>,
    source_map: Option<SharedSourceMap>,
    star_export: bool,
  ) -> Self {
    let resource_identifier =
      create_resource_identifier_for_esm_dependency(&request, attributes.as_ref());
    let loc = range.to_loc(source_map.as_deref());
    Self {
      id: DependencyId::new(),
      source_order,
      request,
      range,
      dependency_type,
      phase,
      attributes,
      resource_identifier,
      loc,
      factorize_info: Default::default(),
      lazy_make: false,
      star_export,
    }
  }

  pub fn set_lazy(&mut self) {
    self.lazy_make = true;
  }
}

pub fn esm_import_dependency_apply<T: ModuleDependency>(
  module_dependency: &T,
  source_order: i32,
  phase: ImportPhase,
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
  let module_graph_cache = &compilation.module_graph_cache_artifact;
  let connection = module_graph.connection_by_dependency_id(module_dependency.id());
  let is_target_active = if let Some(con) = connection {
    con.is_target_active(module_graph, *runtime, module_graph_cache)
  } else {
    true
  };
  // Bailout only if the module does exist and not active.
  if !is_target_active {
    return;
  }

  let target_module = module_graph.get_module_by_dependency_id(module_dependency.id());
  if module_dependency.weak() {
    // lazy
    if target_module.is_none() {
      return;
    }
    // weak
    if let Some(target_module) = target_module
      && ChunkGraph::get_module_id(&compilation.module_ids_artifact, target_module.identifier())
        .is_none()
    {
      return;
    }
  }

  let runtime_condition = if module_dependency.weak() {
    RuntimeCondition::Boolean(false)
  } else if let Some(connection) = module_graph.connection_by_dependency_id(module_dependency.id())
  {
    filter_runtime(*runtime, |r| {
      connection.is_target_active(module_graph, r, module_graph_cache)
    })
  } else {
    RuntimeCondition::Boolean(true)
  };

  let import_var = compilation.get_import_var(
    module.identifier(),
    target_module,
    module_dependency.user_request(),
    phase,
    *runtime,
  );
  let content: (String, String) = compilation.runtime_template.import_statement(
    *module,
    compilation,
    runtime_requirements,
    module_dependency.id(),
    &import_var,
    module_dependency.request(),
    phase,
    false,
  );
  let TemplateContext {
    init_fragments,
    compilation,
    module,
    ..
  } = code_generatable_context;

  // https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/dependencies/HarmonyImportDependency.js#L282-L285
  let module_key = target_module
    .map(|m| m.identifier().as_str())
    .unwrap_or(module_dependency.request());
  let key = format!(
    "{}ESM import {module_key}",
    match phase {
      ImportPhase::Evaluation => "",
      ImportPhase::Source => "",
      ImportPhase::Defer => "deferred ",
    }
  );

  // The import emitted map is consumed by ESMAcceptDependency which enabled by HotModuleReplacementPlugin
  if let Some(import_emitted_map) = import_emitted_runtime::get_map()
    && let Some(target_module) = target_module
  {
    let target_module = target_module.identifier();
    let mut emitted_modules = import_emitted_map.entry(module.identifier()).or_default();

    let old_runtime_condition = match emitted_modules.get(&target_module) {
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
        merged_runtime_condition
          .as_spec_mut()
          .expect("should be spec")
          .extend(old_runtime_condition.as_spec().expect("should be spec"));
      }
    }
    emitted_modules.insert(target_module, merged_runtime_condition);
  }

  let is_async_module = matches!(target_module, Some(target_module) if ModuleGraph::is_async(&compilation.async_modules_artifact.borrow(), &target_module.identifier()));
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
      InitFragmentKey::ESMImport(format!("{key} compat")),
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
  module_graph_cache: &ModuleGraphCacheArtifact,
  name: &Atom,
  is_reexport: bool,
  should_error: bool,
) -> Option<Diagnostic> {
  let imported_module = module_graph.get_module_by_dependency_id(module_dependency.id())?;
  if imported_module.first_error().is_some() {
    return None;
  }
  let parent_module_identifier = module_graph
    .get_parent_module(module_dependency.id())
    .expect("should have parent module for dependency");
  let parent_module = module_graph
    .module_by_identifier(parent_module_identifier)
    .expect("should have module");
  let exports_type = imported_module.get_exports_type(
    module_graph,
    module_graph_cache,
    parent_module.build_meta().strict_esm_module,
  );
  let additional_msg = || {
    if is_reexport {
      format!("(reexported as '{name}')")
    } else {
      format!("(imported as '{name}')")
    }
  };
  let create_error = |message: String| {
    let (severity, title) = if should_error {
      (Severity::Error, "ESModulesLinkingError")
    } else {
      (Severity::Warning, "ESModulesLinkingWarning")
    };
    let mut error = if let Some(span) = module_dependency.range()
      && let Some(source) = parent_module.source()
    {
      Error::from_string(
        Some(source.source().into_string_lossy().into_owned()),
        span.start as usize,
        span.end as usize,
        title.to_string(),
        message,
      )
    } else {
      let mut error = rspack_error::error!(message);
      error.code = Some(title.into());
      error
    };
    error.severity = severity;
    error.hide_stack = Some(true);
    let mut diagnostic = Diagnostic::from(error);
    diagnostic.module_identifier = Some(*parent_module_identifier);
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
    let exports_info = module_graph.get_prefetched_exports_info(
      &imported_module_identifier,
      PrefetchExportsInfoMode::Nested(ids),
    );
    if (!matches!(exports_type, ExportsType::DefaultWithNamed) || ids[0] != "default")
      && matches!(
        exports_info.is_export_provided(ids),
        Some(ExportProvided::NotProvided)
      )
    {
      // For type re-export cases:
      //   1. export { T } from "./types";
      //   2. import { T } from "./types"; export { T };
      // Check if the export is really a type export, if it is, then skip the error.
      let type_reexports_presence = parent_module
        .as_normal_module()
        .and_then(|m| m.get_parser_options())
        .and_then(|o| o.get_javascript())
        .and_then(|o| o.type_reexports_presence)
        .unwrap_or_default();
      // ref: https://github.com/evanw/esbuild/blob/f4159a7b823cd5fe2217da2c30e8873d2f319667/internal/linker/linker.go#L3129-L3131
      if !matches!(
        type_reexports_presence,
        TypeReexportPresenceMode::NoTolerant
      ) && parent_module
        .build_info()
        .collected_typescript_info
        .is_some()
        && ids.len() == 1
        && matches!(
          module_graph
            .get_prefetched_exports_info(parent_module_identifier, PrefetchExportsInfoMode::Default)
            .is_export_provided(std::slice::from_ref(name)),
          Some(ExportProvided::Provided)
        )
      {
        if matches!(
          type_reexports_presence,
          TypeReexportPresenceMode::TolerantNoCheck
        ) {
          return None;
        }
        if find_type_exports_from_outgoings(
          module_graph,
          &imported_module_identifier,
          &ids[0],
          &mut IdentifierSet::default(),
        ) {
          return None;
        }
      }
      let mut pos = 0;
      let mut maybe_exports_info = Some(module_graph.get_prefetched_exports_info(
        &imported_module_identifier,
        PrefetchExportsInfoMode::Nested(ids),
      ));
      while pos < ids.len()
        && let Some(exports_info) = &maybe_exports_info
      {
        let id = &ids[pos];
        pos += 1;
        let export_info = exports_info.get_read_only_export_info(id);
        if matches!(export_info.provided(), Some(ExportProvided::NotProvided)) {
          let provided_exports = exports_info.get_provided_exports();
          let more_info = if let ProvidedExports::ProvidedNames(exports) = &provided_exports {
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
            additional_msg(),
            module_dependency.user_request(),
          );
          return Some(create_error(msg));
        }
        let Some(nested_exports_info) = export_info.exports_info() else {
          maybe_exports_info = None;
          continue;
        };
        maybe_exports_info = Some(exports_info.redirect(nested_exports_info, true));
      }
      let msg = format!(
        "export {} {} was not found in '{}'",
        ids
          .iter()
          .map(|id| format!("'{id}'"))
          .collect::<Vec<_>>()
          .join("."),
        additional_msg(),
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
          additional_msg(),
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
          additional_msg(),
        );
        return Some(create_error(msg));
      }
    }
    _ => {}
  }
  None
}

fn find_type_exports_from_outgoings(
  mg: &ModuleGraph,
  module_identifier: &ModuleIdentifier,
  export_name: &Atom,
  visited: &mut IdentifierSet,
) -> bool {
  visited.insert(*module_identifier);
  let module = mg
    .module_by_identifier(module_identifier)
    .expect("should have module");
  // bailout the check of this export chain if there is a module that not transpiled from
  // typescript, we only support that the export chain is all transpiled typescript, if not
  // the check will be very slow especially when big javascript npm package exists.
  let Some(info) = &module.build_info().collected_typescript_info else {
    return false;
  };
  if info.type_exports.contains(export_name) {
    return true;
  }
  for connection in mg.get_outgoing_connections(module_identifier) {
    if visited.contains(connection.module_identifier()) {
      continue;
    }
    let dependency = mg.dependency_by_id(&connection.dependency_id);
    if !matches!(
      dependency.dependency_type(),
      DependencyType::EsmImport | DependencyType::EsmExportImport
    ) {
      continue;
    }
    if find_type_exports_from_outgoings(mg, connection.module_identifier(), export_name, visited) {
      return true;
    }
  }
  false
}

#[cacheable_dyn]
impl Dependency for ESMImportSideEffectDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn loc(&self) -> Option<DependencyLocation> {
    self.loc.clone()
  }

  fn range(&self) -> Option<DependencyRange> {
    Some(self.range)
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

  fn get_phase(&self) -> ImportPhase {
    self.phase
  }

  fn get_attributes(&self) -> Option<&ImportAttributes> {
    self.attributes.as_ref()
  }

  fn get_module_evaluation_side_effects_state(
    &self,
    module_graph: &ModuleGraph,
    module_graph_cache: &ModuleGraphCacheArtifact,
    module_chain: &mut IdentifierSet,
    connection_state_cache: &mut IdentifierMap<ConnectionState>,
  ) -> ConnectionState {
    if let Some(module) = module_graph
      .module_identifier_by_dependency_id(&self.id)
      .and_then(|module_identifier| module_graph.module_by_identifier(module_identifier))
    {
      module.get_side_effects_connection_state(
        module_graph,
        module_graph_cache,
        module_chain,
        connection_state_cache,
      )
    } else {
      ConnectionState::Active(true)
    }
  }

  fn resource_identifier(&self) -> Option<&str> {
    Some(&self.resource_identifier)
  }

  fn get_referenced_exports(
    &self,
    _module_graph: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
    _runtime: Option<&RuntimeSpec>,
  ) -> Vec<ExtendedReferencedExport> {
    vec![]
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }

  fn forward_id(&self) -> ForwardId {
    ForwardId::Empty
  }

  fn lazy(&self) -> Option<LazyUntil> {
    self.lazy_make.then(|| {
      if self.star_export {
        LazyUntil::Fallback
      } else {
        LazyUntil::NoUntil
      }
    })
  }

  fn unset_lazy(&mut self) -> bool {
    let changed = self.lazy_make;
    self.lazy_make = false;
    changed
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

  fn get_condition(&self) -> Option<DependencyCondition> {
    Some(DependencyCondition::new(
      ESMImportSideEffectDependencyCondition,
    ))
  }

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}

impl AsContextDependency for ESMImportSideEffectDependency {}

struct ESMImportSideEffectDependencyCondition;

impl DependencyConditionFn for ESMImportSideEffectDependencyCondition {
  fn get_connection_state(
    &self,
    conn: &rspack_core::ModuleGraphConnection,
    _runtime: Option<&RuntimeSpec>,
    module_graph: &ModuleGraph,
    module_graph_cache: &ModuleGraphCacheArtifact,
  ) -> ConnectionState {
    let id = *conn.module_identifier();
    if let Some(module) = module_graph.module_by_identifier(&id) {
      module.get_side_effects_connection_state(
        module_graph,
        module_graph_cache,
        &mut IdentifierSet::default(),
        &mut IdentifierMap::default(),
      )
    } else {
      ConnectionState::Active(true)
    }
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for ESMImportSideEffectDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(ESMImportSideEffectDependencyTemplate::template_type())
  }
}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct ESMImportSideEffectDependencyTemplate;

impl ESMImportSideEffectDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::EsmImport)
  }
}

impl DependencyTemplate for ESMImportSideEffectDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    _source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<ESMImportSideEffectDependency>()
      .expect("ESMImportSideEffectDependencyTemplate should only be used for ESMImportSideEffectDependency");
    let TemplateContext {
      compilation,
      concatenation_scope,
      ..
    } = code_generatable_context;
    let module_graph = compilation.get_module_graph();

    let module = module_graph.get_module_by_dependency_id(&dep.id);

    if let Some(module) = module {
      let source_types = module.source_types(module_graph);
      if source_types
        .iter()
        .all(|source_type| matches!(source_type, SourceType::Css))
      {
        return;
      }
    }

    if let Some(scope) = concatenation_scope
      && module.is_some_and(|m| scope.is_module_in_scope(&m.identifier()))
    {
      return;
    }
    esm_import_dependency_apply(dep, dep.source_order, dep.phase, code_generatable_context);
  }
}
