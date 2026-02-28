use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsOption, AsPreset, AsVec},
};
use rspack_core::{
  AsContextDependency, Dependency, DependencyCategory, DependencyCodeGeneration, DependencyId,
  DependencyLocation, DependencyRange, DependencyTemplate, DependencyTemplateType, DependencyType,
  ExportsInfoArtifact, ExtendedReferencedExport, FactorizeInfo, ModuleDependency, ModuleGraph,
  ModuleGraphCacheArtifact, ReferencedExport, RuntimeSpec, TemplateContext, TemplateReplaceSource,
  create_exports_object_referenced,
};
use swc_core::ecma::atoms::Atom;

#[cacheable]
#[derive(Debug, Clone)]
pub struct CommonJsRequireDependency {
  id: DependencyId,
  request: String,
  optional: bool,
  range: DependencyRange,
  range_expr: Option<DependencyRange>,
  loc: Option<DependencyLocation>,
  #[cacheable(with=AsOption<AsVec<AsVec<AsPreset>>>)]
  referenced_exports: Option<Vec<Vec<Atom>>>,
  factorize_info: FactorizeInfo,
}

impl CommonJsRequireDependency {
  pub fn new(
    request: String,
    range: DependencyRange,
    range_expr: Option<DependencyRange>,
    optional: bool,
    loc: Option<DependencyLocation>,
    referenced_exports: Option<Vec<Vec<Atom>>>,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      optional,
      range,
      range_expr,
      loc,
      referenced_exports,
      factorize_info: Default::default(),
    }
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
    _module_graph: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
    _exports_info_artifact: &ExportsInfoArtifact,
    _runtime: Option<&RuntimeSpec>,
  ) -> Vec<ExtendedReferencedExport> {
    if let Some(referenced_exports) = &self.referenced_exports {
      return referenced_exports
        .iter()
        .map(|referenced_export| {
          ExtendedReferencedExport::Export(ReferencedExport::new(
            referenced_export.clone(),
            false,
            false,
          ))
        })
        .collect();
    }
    create_exports_object_referenced()
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
      code_generatable_context
        .runtime_template
        .module_id(
          code_generatable_context.compilation,
          &dep.id,
          &dep.request,
          false,
        )
        .as_str(),
      None,
    );
  }
}
