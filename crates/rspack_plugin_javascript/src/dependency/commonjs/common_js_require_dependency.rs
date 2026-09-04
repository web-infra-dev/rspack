use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsCacheable, AsOption, AsVec},
};
use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::{
  AsContextDependency, ConnectionState, Context, Dependency, DependencyCategory,
  DependencyCodeGeneration, DependencyCondition, DependencyConditionFn, DependencyId,
  DependencyLocation, DependencyRange, DependencyTemplate, DependencyTemplateType, DependencyType,
  ExportsInfoArtifact, ModuleDependency, ModuleGraph, ModuleGraphCacheArtifact,
  ModuleGraphConnection, ReferencedExport, ReferencedSpecifier, ResourceIdentifier, RuntimeSpec,
  SideEffectsStateArtifact, TemplateContext, TemplateReplaceSource, UsedByExports,
  create_exports_object_referenced, create_no_exports_referenced,
  create_referenced_exports_by_referenced_specifiers,
};

use super::create_resource_identifier_for_contextual_commonjs_dependency;
use crate::{
  connection_active_used_by_exports,
  dependency::{DependencyBranchGuard, compose_dependency_condition},
};

#[cacheable]
#[derive(Debug)]
pub struct CommonJsRequireDependency {
  id: DependencyId,
  request: String,
  optional: bool,
  range: DependencyRange,
  range_expr: Option<DependencyRange>,
  loc: Option<DependencyLocation>,
  #[cacheable(with=AsOption<AsVec<AsCacheable>>)]
  referenced_specifiers: Option<Vec<ReferencedSpecifier>>,
  evaluation_only: bool,
  #[cacheable(with=AsOption<AsCacheable>)]
  branch_guard: Option<DependencyBranchGuard>,
  used_by_exports: Option<UsedByExports>,
  context: Option<Context>,
  resource_identifier: ResourceIdentifier,
}

impl CommonJsRequireDependency {
  pub fn new(
    request: String,
    range: DependencyRange,
    range_expr: Option<DependencyRange>,
    optional: bool,
    loc: Option<DependencyLocation>,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      optional,
      range,
      range_expr,
      loc,
      referenced_specifiers: None,
      evaluation_only: false,
      branch_guard: None,
      used_by_exports: None,
      context: None,
      resource_identifier: Default::default(),
    }
  }

  pub fn new_contextual(
    request: String,
    range: DependencyRange,
    range_expr: Option<DependencyRange>,
    optional: bool,
    context: Context,
    loc: Option<DependencyLocation>,
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
      ..Self::new(request, range, range_expr, optional, loc)
    }
  }

  pub fn set_referenced_specifiers(&mut self, referenced_specifiers: Vec<ReferencedSpecifier>) {
    self.evaluation_only = referenced_specifiers.is_empty();
    self.referenced_specifiers = Some(referenced_specifiers);
  }

  pub fn set_evaluation_only(&mut self) {
    self.evaluation_only = true;
    self.referenced_specifiers = Some(Vec::new());
  }

  pub fn is_evaluation_only(&self) -> bool {
    self.evaluation_only
  }

  pub fn set_branch_guard(&mut self, guard: DependencyBranchGuard) {
    self.branch_guard = Some(match self.branch_guard.take() {
      Some(old_guard) => old_guard.and(guard),
      None => guard,
    });
  }

  pub fn set_used_by_exports(&mut self, used_by_exports: Option<UsedByExports>) {
    self.used_by_exports = used_by_exports;
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
  ) -> Vec<ReferencedExport> {
    if self.evaluation_only {
      return create_no_exports_referenced();
    }
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
    let condition = (self.evaluation_only || self.used_by_exports.is_some())
      .then(|| DependencyCondition::new(CommonJsRequireDependencyCondition));
    compose_dependency_condition(condition, self.branch_guard.as_ref())
  }
}

struct CommonJsRequireDependencyCondition;

impl DependencyConditionFn for CommonJsRequireDependencyCondition {
  fn get_connection_state(
    &self,
    connection: &ModuleGraphConnection,
    runtime: Option<&RuntimeSpec>,
    module_graph: &ModuleGraph,
    module_graph_cache: &ModuleGraphCacheArtifact,
    side_effects_state_artifact: &SideEffectsStateArtifact,
    exports_info_artifact: &ExportsInfoArtifact,
  ) -> ConnectionState {
    let dependency = module_graph.dependency_by_id(&connection.dependency_id);
    let dependency = dependency
      .downcast_ref::<CommonJsRequireDependency>()
      .expect("should be CommonJsRequireDependency");
    if !connection_active_used_by_exports(
      connection,
      runtime,
      module_graph,
      exports_info_artifact,
      dependency.used_by_exports.as_ref(),
    ) {
      return ConnectionState::Active(false);
    }
    if !dependency.evaluation_only {
      return ConnectionState::Active(true);
    }

    let module_identifier = *connection.module_identifier();
    if let Some(state) =
      side_effects_state_artifact.module_evaluation_side_effects_state(&module_identifier)
    {
      return state;
    }
    if let Some(module) = module_graph.module_by_identifier(&module_identifier) {
      module.get_side_effects_connection_state(
        module_graph,
        module_graph_cache,
        side_effects_state_artifact,
        &mut IdentifierSet::default(),
        &mut IdentifierMap::default(),
      )
    } else {
      ConnectionState::Active(true)
    }
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for CommonJsRequireDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(CommonJsRequireDependencyTemplate::template_type())
  }
}

impl AsContextDependency for CommonJsRequireDependency {}

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

    if dep.evaluation_only {
      let compilation = code_generatable_context.compilation;
      let module_graph = compilation.get_module_graph();
      if let Some(connection) = module_graph.connection_by_dependency_id(&dep.id)
        && !connection.is_target_active(
          module_graph,
          code_generatable_context.runtime,
          &compilation.module_graph_cache_artifact,
          &compilation
            .build_module_graph_artifact
            .side_effects_state_artifact,
          &compilation.exports_info_artifact,
        )
      {
        if let Some(range) = dep.range_expr {
          source.replace(range.start, range.end, "0".into(), None);
        }
        return;
      }
    }

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
