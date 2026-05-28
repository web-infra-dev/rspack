use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsModuleDependency, DependencyCodeGeneration, DependencyRange,
  DependencyTemplate, DependencyTemplateType, DependencyType, TemplateContext,
  TemplateReplaceSource,
};
use rspack_util::json_stringify_str;

#[cacheable]
#[derive(Debug, Clone)]
pub struct RstestRequireResolveOriginDependency {
  callee_range: DependencyRange,
  args_end: u32,
  origin_path: String,
}

impl RstestRequireResolveOriginDependency {
  pub fn new(callee_range: DependencyRange, args_end: u32, origin_path: String) -> Self {
    Self {
      callee_range,
      args_end,
      origin_path,
    }
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for RstestRequireResolveOriginDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(RstestRequireResolveOriginDependencyTemplate::template_type())
  }
}

impl AsModuleDependency for RstestRequireResolveOriginDependency {}
impl AsContextDependency for RstestRequireResolveOriginDependency {}

#[cacheable]
#[derive(Debug, Clone)]
pub struct RstestRequireResolveOriginDependencyTemplate {
  function_name: String,
}

impl RstestRequireResolveOriginDependencyTemplate {
  pub fn new(function_name: String) -> Self {
    Self { function_name }
  }

  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::RstestRequireResolveOrigin)
  }
}

impl DependencyTemplate for RstestRequireResolveOriginDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    _code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<RstestRequireResolveOriginDependency>()
      .expect(
        "RstestRequireResolveOriginDependencyTemplate can only be applied to \
         RstestRequireResolveOriginDependency",
      );

    source.replace(
      dep.callee_range.start,
      dep.callee_range.end,
      self.function_name.clone(),
      None,
    );

    source.replace(
      dep.args_end,
      dep.args_end,
      format!(", {}", json_stringify_str(&dep.origin_path)),
      None,
    );
  }
}
