use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsCacheable, AsOption, AsPreset, AsVec},
};
use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::{
  AsContextDependency, ConnectionState, Dependency, DependencyCategory, DependencyCodeGeneration,
  DependencyCondition, DependencyConditionFn, DependencyId, DependencyLocation, DependencyRange,
  DependencyTemplate, DependencyTemplateType, DependencyType, ExportPresenceMode, ExportProvided,
  ExportsInfoGetter, ExportsType, ExtendedReferencedExport, FactorizeInfo, ForwardId,
  GetUsedNameParam, ImportAttributes, ImportPhase, JavascriptParserOptions, ModuleDependency,
  ModuleGraph, ModuleGraphCacheArtifact, ModuleGraphConnection, ModuleReferenceOptions,
  PrefetchExportsInfoMode, ReferencedExport, ResourceIdentifier, RuntimeSpec, TemplateContext,
  TemplateReplaceSource, UsedByExports, UsedName, create_exports_object_referenced,
  get_exports_type, property_access, to_normal_comment,
};
use rspack_error::Diagnostic;
use rspack_util::json_stringify;
use swc_core::ecma::atoms::Atom;

use super::{
  create_resource_identifier_for_esm_dependency,
  esm_import_dependency::esm_import_dependency_get_linking_error, esm_import_dependency_apply,
};
use crate::{
  connection_active_inline_value_for_esm_import_specifier, connection_active_used_by_exports,
  is_export_inlined, visitors::DestructuringAssignmentProperties,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct ESMImportSpecifierDependency {
  id: DependencyId,
  #[cacheable(with=AsPreset)]
  request: Atom,
  #[cacheable(with=AsPreset)]
  name: Atom,
  source_order: i32,
  shorthand: bool,
  asi_safe: bool,
  range: DependencyRange,
  #[cacheable(with=AsVec<AsPreset>)]
  ids: Vec<Atom>,
  call: bool,
  direct_import: bool,
  used_by_exports: Option<UsedByExports>,
  #[cacheable(with=AsOption<AsCacheable>)]
  referenced_properties_in_destructuring: Option<DestructuringAssignmentProperties>,
  resource_identifier: ResourceIdentifier,
  export_presence_mode: ExportPresenceMode,
  phase: ImportPhase,
  attributes: Option<ImportAttributes>,
  pub evaluated_in_operator: bool,
  loc: Option<DependencyLocation>,
  pub namespace_object_as_context: bool,
  factorize_info: FactorizeInfo,
}

impl ESMImportSpecifierDependency {
  #[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
  pub fn new(
    request: Atom,
    name: Atom,
    source_order: i32,
    shorthand: bool,
    asi_safe: bool,
    range: DependencyRange,
    ids: Vec<Atom>,
    call: bool,
    direct_import: bool,
    export_presence_mode: ExportPresenceMode,
    referenced_properties_in_destructuring: Option<DestructuringAssignmentProperties>,
    phase: ImportPhase,
    attributes: Option<ImportAttributes>,
    source: Option<&str>,
  ) -> Self {
    let resource_identifier =
      create_resource_identifier_for_esm_dependency(&request, attributes.as_ref());
    let loc = range.to_loc(source);
    Self {
      id: DependencyId::new(),
      request,
      name,
      source_order,
      shorthand,
      asi_safe,
      range,
      ids,
      call,
      direct_import,
      export_presence_mode,
      used_by_exports: None,
      evaluated_in_operator: false,
      namespace_object_as_context: false,
      referenced_properties_in_destructuring,
      phase,
      attributes,
      resource_identifier,
      loc,
      factorize_info: Default::default(),
    }
  }

  pub fn get_ids<'a>(&'a self, mg: &'a ModuleGraph) -> &'a [Atom] {
    mg.get_dep_meta_if_existing(&self.id)
      .map_or_else(|| self.ids.as_slice(), |meta| meta.ids.as_slice())
  }

  // Removed get_esm_import_specifier_referenced_exports

  pub fn get_referenced_exports_in_destructuring(
    &self,
    ids: Option<&[Atom]>,
  ) -> Vec<ExtendedReferencedExport> {
    if let Some(referenced_properties) = &self.referenced_properties_in_destructuring {
      let mut refs = Vec::new();
      referenced_properties.traverse_on_leaf(&mut |stack| {
        let ids_in_destructuring = stack.iter().map(|p| p.id.clone());
        if let Some(ids) = ids {
          let mut ids = ids.to_vec();
          ids.extend(ids_in_destructuring);
          refs.push(ids);
        } else {
          refs.push(ids_in_destructuring.collect::<Vec<_>>());
        }
      });
      refs
        .into_iter()
        // Do not inline if there are any places where used as destructuring
        .map(|name| ExtendedReferencedExport::Export(ReferencedExport::new(name, true, false)))
        .collect::<Vec<_>>()
    } else if let Some(v) = ids {
      vec![ExtendedReferencedExport::Export(ReferencedExport {
        name: v.to_vec(),
        can_mangle: true,
        // Need access the export value to trigger side effects for deferred module
        can_inline: !self.phase.is_defer(),
      })]
    } else {
      create_exports_object_referenced()
    }
  }

  pub fn create_export_presence_mode(options: &JavascriptParserOptions) -> ExportPresenceMode {
    options
      .import_exports_presence
      .or(options.exports_presence)
      .unwrap_or(if let Some(true) = options.strict_export_presence {
        ExportPresenceMode::Error
      } else {
        ExportPresenceMode::Auto
      })
  }

  pub fn set_used_by_exports(&mut self, used_by_exports: Option<UsedByExports>) {
    self.used_by_exports = used_by_exports;
  }
}

#[cacheable_dyn]
impl Dependency for ESMImportSpecifierDependency {
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

  fn get_phase(&self) -> ImportPhase {
    self.phase
  }

  fn get_attributes(&self) -> Option<&ImportAttributes> {
    self.attributes.as_ref()
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::EsmImportSpecifier
  }

  fn get_module_evaluation_side_effects_state(
    &self,
    _module_graph: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
    _module_chain: &mut IdentifierSet,
    _connection_state_cache: &mut IdentifierMap<ConnectionState>,
  ) -> ConnectionState {
    ConnectionState::Active(false)
  }

  fn resource_identifier(&self) -> Option<&str> {
    Some(&self.resource_identifier)
  }

  // #[tracing::instrument(skip_all)]
  fn get_diagnostics(
    &self,
    module_graph: &ModuleGraph,
    module_graph_cache: &ModuleGraphCacheArtifact,
  ) -> Option<Vec<Diagnostic>> {
    let module = module_graph.get_parent_module(&self.id)?;
    let module = module_graph.module_by_identifier(module)?;
    if let Some(should_error) = self
      .export_presence_mode
      .get_effective_export_presence(module.as_ref())
      && let Some(diagnostic) = esm_import_dependency_get_linking_error(
        self,
        self.get_ids(module_graph),
        module_graph,
        module_graph_cache,
        &self.name,
        false,
        should_error,
      )
    {
      return Some(vec![diagnostic]);
    }
    None
  }

  fn get_referenced_exports(
    &self,
    module_graph: &ModuleGraph,
    module_graph_cache: &ModuleGraphCacheArtifact,
    _runtime: Option<&RuntimeSpec>,
  ) -> Vec<ExtendedReferencedExport> {
    let ids = self.get_ids(module_graph);
    if ids.is_empty() {
      return self.get_referenced_exports_in_destructuring(None);
    }

    let exports_type = if let Some(id) = ids.first()
      && id == "default"
    {
      let parent_module = module_graph
        .get_parent_module(&self.id)
        .expect("should have parent module");
      Some(get_exports_type(
        module_graph,
        module_graph_cache,
        &self.id,
        parent_module,
      ))
    } else {
      None
    };

    let mut ids = ids;
    let mut namespace_object_as_context = self.namespace_object_as_context;
    if let Some(id) = ids.first()
      && id == "default"
    {
      match exports_type {
        Some(ExportsType::DefaultOnly | ExportsType::DefaultWithNamed) => {
          if ids.len() == 1 {
            return self.get_referenced_exports_in_destructuring(None);
          }
          ids = &ids[1..];
          namespace_object_as_context = true;
        }
        Some(ExportsType::Dynamic) => {
          return create_exports_object_referenced();
        }
        _ => {}
      }
    }

    if self.call && !self.direct_import && (namespace_object_as_context || ids.len() > 1) {
      if ids.len() == 1 {
        return create_exports_object_referenced();
      }
      // remove last one
      ids = &ids[..ids.len() - 1];
    }
    self.get_referenced_exports_in_destructuring(Some(ids))
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }

  fn forward_id(&self) -> ForwardId {
    if let Some(id) = self.ids.first() {
      ForwardId::Id(id.clone())
    } else {
      ForwardId::All
    }
  }
}

#[cacheable_dyn]
impl ModuleDependency for ESMImportSpecifierDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn get_condition(&self) -> Option<DependencyCondition> {
    Some(DependencyCondition::new(
      ESMImportSpecifierDependencyCondition,
    ))
  }

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}

impl AsContextDependency for ESMImportSpecifierDependency {}

#[cacheable_dyn]
impl DependencyCodeGeneration for ESMImportSpecifierDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(ESMImportSpecifierDependencyTemplate::template_type())
  }
}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct ESMImportSpecifierDependencyTemplate;

impl ESMImportSpecifierDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::EsmImportSpecifier)
  }

  fn get_code_for_ids(
    &self,
    ids: &[Atom],
    dep: &ESMImportSpecifierDependency,
    connection: Option<&ModuleGraphConnection>,
    code_generatable_context: &mut TemplateContext,
  ) -> String {
    let TemplateContext {
      concatenation_scope,
      ..
    } = code_generatable_context;
    if let Some(scope) = concatenation_scope
      && let Some(con) = connection
      && scope.is_module_in_scope(con.module_identifier())
    {
      if ids.is_empty() {
        scope.create_module_reference(
          con.module_identifier(),
          &ModuleReferenceOptions {
            asi_safe: Some(dep.asi_safe),
            deferred_import: dep.phase.is_defer(),
            ..Default::default()
          },
        )
      } else if dep.namespace_object_as_context && ids.len() == 1 {
        // ConcatenationScope::create_module_reference(&dep, module, options)
        scope.create_module_reference(
          con.module_identifier(),
          &ModuleReferenceOptions {
            asi_safe: Some(dep.asi_safe),
            deferred_import: dep.phase.is_defer(),
            ..Default::default()
          },
        ) + property_access(ids, 0).as_str()
      } else {
        scope.create_module_reference(
          con.module_identifier(),
          &ModuleReferenceOptions {
            asi_safe: Some(dep.asi_safe),
            ids: ids.to_vec(),
            call: dep.call,
            direct_import: dep.direct_import,
            deferred_import: dep.phase.is_defer(),
            ..Default::default()
          },
        )
      }
    } else {
      let mg = code_generatable_context.compilation.get_module_graph();
      let target_module = mg.get_module_by_dependency_id(&dep.id);
      let import_var = code_generatable_context.compilation.get_import_var(
        code_generatable_context.module.identifier(),
        target_module,
        dep.user_request(),
        dep.phase,
        code_generatable_context.runtime,
      );
      esm_import_dependency_apply(dep, dep.source_order, dep.phase, code_generatable_context);
      let mut new_init_fragment = vec![];
      let res = code_generatable_context
        .runtime_template
        .export_from_import(
          code_generatable_context.compilation,
          &mut new_init_fragment,
          code_generatable_context.module.identifier(),
          code_generatable_context.runtime,
          true,
          &dep.request,
          &import_var,
          ids,
          &dep.id,
          dep.call,
          !dep.direct_import,
          Some(dep.shorthand || dep.asi_safe),
          dep.phase,
        );
      code_generatable_context
        .init_fragments
        .extend(new_init_fragment);
      res
    }
  }

  fn render_evaluated_in_operator(
    &self,
    ids: &[Atom],
    dep: &ESMImportSpecifierDependency,
    connection: Option<&ModuleGraphConnection>,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let Some(con) = connection else {
      return;
    };
    let TemplateContext {
      compilation,
      runtime,
      module: self_module,
      ..
    } = code_generatable_context;
    let mg = compilation.get_module_graph();
    let Some(module) = mg.get_module_by_dependency_id(&dep.id) else {
      return;
    };
    let exports_info = mg.get_prefetched_exports_info(
      con.module_identifier(),
      PrefetchExportsInfoMode::Nested(ids),
    );
    let exports_type = module.get_exports_type(
      mg,
      &code_generatable_context
        .compilation
        .module_graph_cache_artifact,
      self_module.build_meta().strict_esm_module,
    );
    let first = ids
      .first()
      .expect("Empty ids is not possible for evaluated in operator esm import specifier");
    let value = match exports_type {
      ExportsType::DefaultWithNamed => {
        if first == "default" {
          if ids.len() == 1 {
            Some(ExportProvided::Provided)
          } else {
            exports_info.is_export_provided(&ids[1..])
          }
        } else {
          exports_info.is_export_provided(ids)
        }
      }
      ExportsType::Namespace => {
        if first == "__esModule" {
          if ids.len() == 1 {
            Some(ExportProvided::Provided)
          } else {
            None
          }
        } else {
          exports_info.is_export_provided(ids)
        }
      }
      ExportsType::Dynamic => {
        if first != "default" {
          exports_info.is_export_provided(ids)
        } else {
          None
        }
      }
      ExportsType::DefaultOnly => None,
    };
    match value {
      Some(ExportProvided::Provided) => {
        source.replace(dep.range.start, dep.range.end, " true", None);
      }
      Some(ExportProvided::NotProvided) => {
        source.replace(dep.range.start, dep.range.end, " false", None)
      }
      _ => {
        let Some(used_name) = ExportsInfoGetter::get_used_name(
          GetUsedNameParam::WithNames(&exports_info),
          *runtime,
          ids,
        )
        .and_then(|used_name| match used_name {
          UsedName::Normal(names) => names.last().cloned(),
          UsedName::Inlined(_) => unreachable!("Inlined must be provided"),
        }) else {
          return;
        };
        let code = self.get_code_for_ids(
          &ids[..(ids.len() - 1)],
          dep,
          connection,
          code_generatable_context,
        );
        source.replace(
          dep.range.start,
          dep.range.end,
          &format!("{} in {code}", json_stringify(&used_name)),
          None,
        )
      }
    }
  }
}

impl DependencyTemplate for ESMImportSpecifierDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<ESMImportSpecifierDependency>()
      .expect(
        "ESMImportSpecifierDependencyTemplate should only be used for ESMImportSpecifierDependency",
      );
    let TemplateContext {
      compilation,
      runtime,
      ..
    } = code_generatable_context;
    let module_graph = compilation.get_module_graph();
    let ids = dep.get_ids(module_graph);
    let connection = module_graph.connection_by_dependency_id(&dep.id);
    // Early return if target is not active and export is not inlined
    if let Some(con) = connection
      && !con.is_target_active(
        module_graph,
        *runtime,
        &compilation.module_graph_cache_artifact,
      )
      && !is_export_inlined(module_graph, con.module_identifier(), ids, *runtime)
    {
      return;
    }

    if dep.evaluated_in_operator {
      return self.render_evaluated_in_operator(
        ids,
        dep,
        connection,
        source,
        code_generatable_context,
      );
    }

    let export_expr = self.get_code_for_ids(ids, dep, connection, code_generatable_context);

    if dep.shorthand {
      source.insert(dep.range.end, format!(": {export_expr}").as_str(), None);
    } else {
      source.replace(dep.range.start, dep.range.end, export_expr.as_str(), None);
    }

    let module_graph = code_generatable_context.compilation.get_module_graph();
    if let Some(referenced_properties) = &dep.referenced_properties_in_destructuring {
      let mut prefixed_ids = ids.to_vec();

      let Some(module) = module_graph.get_module_by_dependency_id(&dep.id) else {
        return;
      };

      if ids.first().is_some_and(|id| id == "default") {
        let self_module = module_graph
          .get_parent_module(&dep.id)
          .and_then(|id| module_graph.module_by_identifier(id))
          .expect("should have parent module");
        let exports_type = module.get_exports_type(
          module_graph,
          &code_generatable_context
            .compilation
            .module_graph_cache_artifact,
          self_module.build_meta().strict_esm_module,
        );
        if matches!(
          exports_type,
          ExportsType::DefaultOnly | ExportsType::DefaultWithNamed
        ) && !ids.is_empty()
        {
          prefixed_ids = ids[1..].to_vec();
        }
      }

      referenced_properties.traverse_on_enter(&mut |stack| {
        let prop = stack.last().expect("should have last");
        let mut concated_ids = prefixed_ids.clone();
        concated_ids.extend(stack.iter().map(|p| p.id.clone()));
        let Some(new_name) = ExportsInfoGetter::get_used_name(
          GetUsedNameParam::WithNames(&module_graph.get_prefetched_exports_info(
            &module.identifier(),
            PrefetchExportsInfoMode::Nested(&concated_ids),
          )),
          code_generatable_context.runtime,
          &concated_ids,
        )
        .and_then(|used| match used {
          UsedName::Normal(names) => names.last().cloned(),
          UsedName::Inlined(inlined) => {
            unreachable!("should not inline for destructuring {:#?}", inlined)
          }
        }) else {
          return;
        };

        if new_name == prop.id {
          return;
        }

        let comment = to_normal_comment(prop.id.as_str());
        let key = format!("{comment}{new_name}");
        let content = if prop.shorthand {
          format!("{key}: {}", prop.id)
        } else {
          key
        };
        source.replace(prop.range.start, prop.range.end, &content, None);
      });
    }
  }
}

struct ESMImportSpecifierDependencyCondition;

impl DependencyConditionFn for ESMImportSpecifierDependencyCondition {
  fn get_connection_state(
    &self,
    connection: &ModuleGraphConnection,
    runtime: Option<&RuntimeSpec>,
    module_graph: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
  ) -> ConnectionState {
    let dependency = module_graph.dependency_by_id(&connection.dependency_id);
    let dependency = dependency
      .downcast_ref::<ESMImportSpecifierDependency>()
      .expect("should be ESMImportSpecifierDependency");
    ConnectionState::Active(
      connection_active_inline_value_for_esm_import_specifier(
        dependency,
        connection,
        runtime,
        module_graph,
      ) && connection_active_used_by_exports(
        connection,
        runtime,
        module_graph,
        dependency.used_by_exports.as_ref(),
      ),
    )
  }
}
