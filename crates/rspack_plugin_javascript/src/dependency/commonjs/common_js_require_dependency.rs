use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsCacheable, AsOption, AsVec},
};
use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::{
  AsContextDependency, Context, Dependency, DependencyCategory, DependencyCodeGeneration,
  DependencyCondition, DependencyConditionFn, DependencyId, DependencyLocation, DependencyRange,
  DependencyTemplate, DependencyTemplateType, DependencyType, ExportsInfoArtifact,
  ExtendedReferencedExport, FactorizeInfo, ModuleDependency, ModuleGraph, ModuleGraphCacheArtifact,
  ModuleGraphConnection, ReferencedSpecifier, ResourceIdentifier, RuntimeGlobals, RuntimeSpec,
  SideEffectsStateArtifact, TemplateContext, TemplateReplaceSource,
  create_exports_object_referenced, create_referenced_exports_by_referenced_specifiers,
};

use super::create_resource_identifier_for_contextual_commonjs_dependency;
use crate::dependency::{DependencyBranchGuard, compose_dependency_condition};

#[cacheable]
#[derive(Debug, Clone)]
pub struct CommonJsRequireDependency {
  id: DependencyId,
  request: String,
  optional: bool,
  range: DependencyRange,
  range_expr: Option<DependencyRange>,
  loc: Option<DependencyLocation>,
  #[cacheable(with=AsOption<AsVec<AsCacheable>>)]
  referenced_specifiers: Option<Vec<ReferencedSpecifier>>,
  #[cacheable(with=AsOption<AsCacheable>)]
  branch_guard: Option<DependencyBranchGuard>,
  context: Option<Context>,
  resource_identifier: ResourceIdentifier,
  factorize_info: FactorizeInfo,
  replace_call: bool,
  call_new: bool,
}

impl CommonJsRequireDependency {
  pub fn new(
    request: String,
    range: DependencyRange,
    range_expr: Option<DependencyRange>,
    optional: bool,
    loc: Option<DependencyLocation>,
    referenced_specifiers: Option<Vec<ReferencedSpecifier>>,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      optional,
      range,
      range_expr,
      loc,
      referenced_specifiers,
      branch_guard: None,
      context: None,
      resource_identifier: Default::default(),
      factorize_info: Default::default(),
      replace_call: false,
      call_new: false,
    }
  }

  pub fn new_contextual(
    request: String,
    range: DependencyRange,
    range_expr: Option<DependencyRange>,
    optional: bool,
    context: Context,
    loc: Option<DependencyLocation>,
    referenced_specifiers: Option<Vec<ReferencedSpecifier>>,
  ) -> Self {
    let resource_identifier = create_resource_identifier_for_contextual_commonjs_dependency(
      "cjs require",
      &context,
      &request,
    )
    .into();
    Self {
      context: Some(context),
      resource_identifier,
      ..Self::new(
        request,
        range,
        range_expr,
        optional,
        loc,
        referenced_specifiers,
      )
    }
  }

  pub fn set_referenced_specifiers(&mut self, referenced_specifiers: Vec<ReferencedSpecifier>) {
    self.referenced_specifiers = Some(referenced_specifiers);
  }

  pub fn set_replace_call(&mut self) {
    self.replace_call = true;
  }

  pub fn set_call_new(&mut self) {
    self.call_new = true;
  }

  pub fn set_branch_guard(&mut self, guard: DependencyBranchGuard) {
    self.branch_guard = Some(match self.branch_guard.take() {
      Some(old_guard) => old_guard.and(guard),
      None => guard,
    });
  }

  fn unused_require_can_be_removed(&self) -> bool {
    self.replace_call
      && self
        .referenced_specifiers
        .as_ref()
        .is_some_and(Vec::is_empty)
  }
}

#[cacheable_dyn]
impl Dependency for CommonJsRequireDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn loc(&self) -> Option<DependencyLocation> {
    self.loc.clone()
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CjsRequire
  }

  fn get_context(&self) -> Option<&Context> {
    self.context.as_ref()
  }

  fn resource_identifier(&self) -> Option<&str> {
    self
      .context
      .as_ref()
      .map(|_| self.resource_identifier.as_str())
  }

  fn range(&self) -> Option<DependencyRange> {
    self.range_expr
  }

  fn get_referenced_exports(
    &self,
    module_graph: &ModuleGraph,
    module_graph_cache: &ModuleGraphCacheArtifact,
    exports_info_artifact: &ExportsInfoArtifact,
    _runtime: Option<&RuntimeSpec>,
  ) -> Vec<ExtendedReferencedExport> {
    if let Some(referenced_specifiers) = &self.referenced_specifiers {
      let module = module_graph
        .get_module_by_dependency_id(&self.id)
        .expect("should have module");
      let exports_type = module.get_exports_type(
        module_graph,
        module_graph_cache,
        exports_info_artifact,
        false,
      );
      create_referenced_exports_by_referenced_specifiers(
        referenced_specifiers,
        exports_type,
        module.build_info().json_data.is_some(),
      )
    } else {
      create_exports_object_referenced()
    }
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }

  fn get_module_evaluation_side_effects_state(
    &self,
    module_graph: &ModuleGraph,
    module_graph_cache: &ModuleGraphCacheArtifact,
    side_effects_state_artifact: &SideEffectsStateArtifact,
    module_chain: &mut IdentifierSet,
    connection_state_cache: &mut IdentifierMap<rspack_core::ConnectionState>,
  ) -> rspack_core::ConnectionState {
    if let Some(module) = module_graph
      .module_identifier_by_dependency_id(&self.id)
      .and_then(|module_identifier| module_graph.module_by_identifier(module_identifier))
    {
      module.get_side_effects_connection_state(
        module_graph,
        module_graph_cache,
        side_effects_state_artifact,
        module_chain,
        connection_state_cache,
      )
    } else {
      rspack_core::ConnectionState::Active(true)
    }
  }
}

#[cacheable_dyn]
impl ModuleDependency for CommonJsRequireDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn get_optional(&self) -> bool {
    self.optional
  }

  fn get_condition(&self) -> Option<DependencyCondition> {
    let base = self
      .unused_require_can_be_removed()
      .then(|| DependencyCondition::new(CommonJsRequireDependencyCondition));
    compose_dependency_condition(base, self.branch_guard.as_ref())
  }

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for CommonJsRequireDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(CommonJsRequireDependencyTemplate::template_type())
  }
}

impl AsContextDependency for CommonJsRequireDependency {}

struct CommonJsRequireDependencyCondition;

impl DependencyConditionFn for CommonJsRequireDependencyCondition {
  fn get_connection_state(
    &self,
    conn: &ModuleGraphConnection,
    _runtime: Option<&RuntimeSpec>,
    module_graph: &ModuleGraph,
    module_graph_cache: &ModuleGraphCacheArtifact,
    side_effects_state_artifact: &SideEffectsStateArtifact,
    _exports_info_artifact: &ExportsInfoArtifact,
  ) -> rspack_core::ConnectionState {
    let id = *conn.module_identifier();
    if let Some(state) = side_effects_state_artifact.module_evaluation_side_effects_state(&id) {
      return state;
    }
    if let Some(module) = module_graph.module_by_identifier(&id) {
      module.get_side_effects_connection_state(
        module_graph,
        module_graph_cache,
        side_effects_state_artifact,
        &mut IdentifierSet::default(),
        &mut IdentifierMap::default(),
      )
    } else {
      rspack_core::ConnectionState::Active(true)
    }
  }
}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct CommonJsRequireDependencyTemplate;

impl CommonJsRequireDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::CjsRequire)
  }
}

impl DependencyTemplate for CommonJsRequireDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<CommonJsRequireDependency>()
      .expect(
        "CommonJsRequireDependencyTemplate should only be used for CommonJsRequireDependency",
      );

    let compilation = code_generatable_context.compilation;
    let module_graph = compilation.get_module_graph();
    let is_target_active = module_graph
      .connection_by_dependency_id(&dep.id)
      .is_none_or(|connection| {
        connection.is_target_active(
          module_graph,
          code_generatable_context.runtime,
          &compilation.module_graph_cache_artifact,
          &compilation
            .build_module_graph_artifact
            .side_effects_state_artifact,
          &compilation.exports_info_artifact,
        )
      });
    if dep.replace_call
      && let Some(range_expr) = dep.range_expr
    {
      let content = if is_target_active {
        let require_call = format!(
          "{}({})",
          code_generatable_context
            .runtime_template
            .render_runtime_globals(&RuntimeGlobals::REQUIRE),
          code_generatable_context.runtime_template.module_id(
            code_generatable_context.compilation,
            &dep.id,
            &dep.request,
            false,
          )
        );
        if dep.call_new {
          format!("new {require_call}")
        } else {
          require_call
        }
      } else {
        "(/* unused require call */ {})".to_string()
      };
      source.replace(range_expr.start, range_expr.end, content, None);
    } else {
      source.replace(
        dep.range.start,
        dep.range.end,
        code_generatable_context.runtime_template.module_id(
          code_generatable_context.compilation,
          &dep.id,
          &dep.request,
          false,
        ),
        None,
      );
    }
  }
}
