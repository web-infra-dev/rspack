use rspack_core::{
  create_exports_object_referenced, export_from_import, get_dependency_used_by_exports_condition,
  get_exports_type, AsContextDependency, ConnectionState, Dependency, DependencyCategory,
  DependencyCondition, DependencyId, DependencyTemplate, DependencyType, ExportPresenceMode,
  ExportsType, ExtendedReferencedExport, JavascriptParserOptions, ModuleDependency, ModuleGraph,
  ModuleIdentifier, ReferencedExport, RuntimeSpec, TemplateContext, TemplateReplaceSource,
  UsedByExports,
};
use rspack_core::{property_access, ModuleReferenceOptions};
use rspack_error::Diagnostic;
use rustc_hash::FxHashSet as HashSet;
use swc_core::{common::Span, ecma::atoms::Atom};

use super::harmony_import_dependency::harmony_import_dependency_get_linking_error;
use super::{create_resource_identifier_for_esm_dependency, harmony_import_dependency_apply};

#[derive(Debug, Clone)]
pub struct HarmonyImportSpecifierDependency {
  id: DependencyId,
  request: Atom,
  name: Atom,
  source_order: i32,
  shorthand: bool,
  asi_safe: bool,
  start: u32,
  end: u32,
  ids: Vec<Atom>,
  call: bool,
  direct_import: bool,
  used_by_exports: Option<UsedByExports>,
  pub namespace_object_as_context: bool,
  referenced_properties_in_destructuring: Option<HashSet<Atom>>,
  resource_identifier: String,
  span_for_on_usage_search: Span,
  export_presence_mode: ExportPresenceMode,
}

impl HarmonyImportSpecifierDependency {
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    request: Atom,
    name: Atom,
    source_order: i32,
    shorthand: bool,
    asi_safe: bool,
    start: u32,
    end: u32,
    ids: Vec<Atom>,
    call: bool,
    direct_import: bool,
    export_presence_mode: ExportPresenceMode,
    referenced_properties_in_destructuring: Option<HashSet<Atom>>,
    span_for_on_usage_search: Span,
  ) -> Self {
    let resource_identifier = create_resource_identifier_for_esm_dependency(&request);
    Self {
      id: DependencyId::new(),
      request,
      name,
      source_order,
      shorthand,
      asi_safe,
      start,
      end,
      ids,
      call,
      direct_import,
      export_presence_mode,
      used_by_exports: None,
      namespace_object_as_context: false,
      referenced_properties_in_destructuring,
      resource_identifier,
      span_for_on_usage_search,
    }
  }

  pub fn get_referenced_exports_in_destructuring(
    &self,
    ids: Option<&Vec<Atom>>,
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
      vec![ExtendedReferencedExport::Array(v.clone())]
    } else {
      create_exports_object_referenced()
    }
  }

  pub fn create_export_presence_mode(options: &JavascriptParserOptions) -> ExportPresenceMode {
    options
      .import_exports_presence
      .or(options.exports_presence)
      .unwrap_or(if options.strict_export_presence {
        ExportPresenceMode::Error
      } else {
        ExportPresenceMode::Auto
      })
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
      runtime,
      concatenation_scope,
      ..
    } = code_generatable_context;
    let module_graph = compilation.get_module_graph();
    // Only available when module factorization is successful.
    let reference_mgm = module_graph.module_graph_module_by_dependency_id(&self.id);
    let connection = module_graph.connection_by_dependency(&self.id);
    let is_target_active = if let Some(con) = connection {
      con.is_target_active(&module_graph, *runtime)
    } else {
      true
    };

    if !is_target_active {
      return;
    }

    let used = reference_mgm.is_some();
    if reference_mgm.is_some() && !used {
      // TODO do this by PureExpressionDependency.
      let value = format!("/* \"{}\" unused */null", self.request);
      if self.shorthand {
        source.insert(self.end, &format!(": {value}"), None);
      } else {
        source.replace(self.start, self.end, &value, None)
      }
      return;
    }

    let ids = self.get_ids(&module_graph);
    let import_var = compilation.get_import_var(&self.id);

    let export_expr = if let Some(scope) = concatenation_scope
      && let Some(con) = module_graph.connection_by_dependency(&self.id)
      && scope.is_module_in_scope(con.module_identifier())
    {
      if ids.is_empty() {
        scope.create_module_reference(
          con.module_identifier(),
          &ModuleReferenceOptions {
            asi_safe: Some(self.asi_safe),
            ..Default::default()
          },
        )
      } else if self.namespace_object_as_context && ids.len() == 1 {
        // ConcatenationScope::create_module_reference(&self, module, options)
        scope.create_module_reference(
          con.module_identifier(),
          &ModuleReferenceOptions {
            asi_safe: Some(self.asi_safe),
            ..Default::default()
          },
        ) + property_access(ids, 0).as_str()
      } else {
        scope.create_module_reference(
          con.module_identifier(),
          &ModuleReferenceOptions {
            asi_safe: Some(self.asi_safe),
            ids,
            call: self.call,
            direct_import: self.direct_import,
            ..Default::default()
          },
        )
      }
    } else {
      harmony_import_dependency_apply(self, self.source_order, code_generatable_context);
      // dbg!(&self.shorthand, self.asi_safe);
      export_from_import(
        code_generatable_context,
        true,
        &self.request,
        &import_var,
        ids,
        &self.id,
        self.call,
        !self.direct_import,
        Some(self.shorthand || self.asi_safe),
      )
    };

    if self.shorthand {
      source.insert(self.end, format!(": {export_expr}").as_str(), None);
    } else {
      source.replace(self.start, self.end, export_expr.as_str(), None);
    }
  }

  fn dependency_id(&self) -> Option<DependencyId> {
    Some(self.id)
  }
}

impl Dependency for HarmonyImportSpecifierDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn source_order(&self) -> Option<i32> {
    Some(self.source_order)
  }
  fn span(&self) -> Option<rspack_core::ErrorSpan> {
    Some(rspack_core::ErrorSpan {
      start: self.start,
      end: self.end,
    })
  }

  fn span_for_on_usage_search(&self) -> Option<rspack_core::ErrorSpan> {
    Some(self.span_for_on_usage_search.into())
  }

  fn set_used_by_exports(&mut self, used_by_exports: Option<UsedByExports>) {
    self.used_by_exports = used_by_exports;
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
    _module_chain: &mut HashSet<ModuleIdentifier>,
  ) -> ConnectionState {
    ConnectionState::Bool(false)
  }

  fn get_ids(&self, mg: &ModuleGraph) -> Vec<Atom> {
    mg.get_dep_meta_if_existing(&self.id)
      .map(|meta| meta.ids.clone())
      .unwrap_or_else(|| self.ids.clone())
  }

  fn resource_identifier(&self) -> Option<&str> {
    Some(&self.resource_identifier)
  }

  fn get_diagnostics(&self, module_graph: &ModuleGraph) -> Option<Vec<Diagnostic>> {
    let module = module_graph.get_parent_module(&self.id)?;
    let module = module_graph.module_by_identifier(module)?;
    if let Some(should_error) = self
      .export_presence_mode
      .get_effective_export_presence(&**module)
      && let Some(diagnostic) = harmony_import_dependency_get_linking_error(
        self,
        &self.get_ids(module_graph)[..],
        module_graph,
        format!("(imported as '{}')", self.name),
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
    _runtime: Option<&RuntimeSpec>,
  ) -> Vec<ExtendedReferencedExport> {
    let mut ids = self.get_ids(module_graph);
    // namespace import
    if ids.is_empty() {
      return self.get_referenced_exports_in_destructuring(None);
    }

    let mut namespace_object_as_context = self.namespace_object_as_context;
    if let Some(id) = ids.first()
      && id == "default"
    {
      let parent_module = module_graph
        .parent_module_by_dependency_id(&self.id)
        .expect("should have parent module");
      let exports_type = get_exports_type(module_graph, &self.id, &parent_module);
      match exports_type {
        ExportsType::DefaultOnly | ExportsType::DefaultWithNamed => {
          if ids.len() == 1 {
            return self.get_referenced_exports_in_destructuring(None);
          }
          ids.drain(0..1);
          namespace_object_as_context = true;
        }
        ExportsType::Dynamic => {
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
      ids.shrink_to(ids.len() - 1);
    }

    self.get_referenced_exports_in_destructuring(Some(&ids))
  }
}

impl ModuleDependency for HarmonyImportSpecifierDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn set_request(&mut self, request: String) {
    self.request = request.into();
  }

  fn get_condition(&self) -> Option<DependencyCondition> {
    // TODO: this part depend on inner graph parser plugin to call set_used_by_exports to update the used_by_exports
    get_dependency_used_by_exports_condition(self.id, self.used_by_exports.as_ref())
  }
}

impl AsContextDependency for HarmonyImportSpecifierDependency {}
