use rspack_cacheable::{cacheable, cacheable_dyn, with::Skip};
use rspack_core::{
  module_id, AsContextDependency, Dependency, DependencyCategory, DependencyCodeGeneration,
  DependencyId, DependencyLocation, DependencyRange, DependencyTemplate, DependencyTemplateType,
  DependencyType, FactorizeInfo, ModuleDependency, SharedSourceMap, TemplateContext,
  TemplateReplaceSource,
};
use rspack_plugin_javascript::dependency::CommonJsRequireDependency;

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
      dep.range.end - 1,
      module_id(
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
