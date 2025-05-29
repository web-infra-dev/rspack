use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsModuleDependency, DependencyCodeGeneration, DependencyTemplate,
  DependencyTemplateType, DependencyType, InitFragmentExt, InitFragmentKey, InitFragmentStage,
  Module, NormalInitFragment, TemplateContext, TemplateReplaceSource,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct MockHoistDependency {}

impl MockHoistDependency {
  pub fn new() -> Self {
    Self {}
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for MockHoistDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(MockHoistDependencyTemplate::template_type())
  }
}

impl AsModuleDependency for MockHoistDependency {}
impl AsContextDependency for MockHoistDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct MockHoistDependencyTemplate;

impl MockHoistDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::RstestHoistMock)
  }
}

impl DependencyTemplate for MockHoistDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    _source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let TemplateContext {
      module,
      init_fragments,
      ..
    } = code_generatable_context;

    let m = module.as_normal_module();
    if let Some(m) = m {
      let resource_path = &m.resource_resolved_data().resource_path;

      let dep = dep
        .as_any()
        .downcast_ref::<MockHoistDependency>()
        .expect("ModulePathNameDependencyTemplate can only be applied to ModulePathNameDependency");

      if let Some(resource_path) = resource_path {
        let init = NormalInitFragment::new(
          format!("// MOCK: const __filename = '{}';\n", resource_path),
          InitFragmentStage::StageConstants,
          0,
          InitFragmentKey::Const(format!("retest mock_hoist {}", m.id())),
          None,
        );

        init_fragments.push(init.boxed());
      }
    }
  }
}
