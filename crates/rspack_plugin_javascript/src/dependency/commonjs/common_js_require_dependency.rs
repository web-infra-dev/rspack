use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsCacheable, AsOption, AsVec},
};
use rspack_core::{
  AsContextDependency, Context, Dependency, DependencyCategory, DependencyCodeGeneration,
  DependencyCondition, DependencyId, DependencyLocation, DependencyRange, DependencyTemplate,
  DependencyTemplateType, DependencyType, ExportsInfoArtifact, ExtendedReferencedExport,
  FactorizeInfo, ModuleDependency, ModuleGraph, ModuleGraphCacheArtifact, ReferencedSpecifier,
  ResourceIdentifier, RuntimeSpec, TemplateContext, TemplateReplaceSource,
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
      branch_guard: None,
      context: None,
      resource_identifier: Default::default(),
      factorize_info: Default::default(),
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
    if referenced_specifiers.is_empty() {
      // If the referenced specifiers are empty, keep it as default (None), since this dependency can't eliminate by side effects optimization,
      // so if we set it to Some(vec![]), and the dependency still executes, it will cause runtime error because the exports are all tree shaken.
      // see test case `tests/rspack-test/configCases/cjs-tree-shaking/side-effects-free`
      return;
    }
    self.referenced_specifiers = Some(referenced_specifiers);
  }

  pub fn set_branch_guard(&mut self, guard: DependencyBranchGuard) {
    self.branch_guard = Some(match self.branch_guard.take() {
      Some(old_guard) => old_guard.and(guard),
      None => guard,
    });
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
    compose_dependency_condition(None, self.branch_guard.as_ref())
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
