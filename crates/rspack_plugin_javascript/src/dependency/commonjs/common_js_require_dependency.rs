use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsCacheable, AsOption, AsVec},
};
use rspack_core::{
  AsContextDependency, Dependency, DependencyCategory, DependencyCodeGeneration, DependencyId,
  DependencyLocation, DependencyRange, DependencyTemplate, DependencyTemplateType, DependencyType,
  ExportsInfoArtifact, ExtendedReferencedExport, FactorizeInfo, ModuleDependency, ModuleGraph,
  ModuleGraphCacheArtifact, ReferencedSpecifier, RuntimeSpec, TemplateContext,
  TemplateReplaceSource, create_exports_object_referenced,
  create_referenced_exports_by_referenced_specifiers,
};

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
  factorize_info: FactorizeInfo,
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
      factorize_info: Default::default(),
    }
  }

  pub fn set_referenced_specifiers(&mut self, referenced_specifiers: Vec<ReferencedSpecifier>) {
    self.referenced_specifiers = Some(referenced_specifiers);
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
