use rspack_cacheable::{cacheable, cacheable_dyn, with::Skip};
use rspack_core::{
  AsContextDependency, AsModuleDependency, DependencyCodeGeneration, DependencyRange,
  DependencyTemplate, DependencyTemplateType, DependencyType, InitFragmentExt, InitFragmentKey,
  InitFragmentStage, NormalInitFragment, TemplateContext, TemplateReplaceSource,
};
use swc_core::common::Span;

#[cacheable]
#[derive(Debug, Clone)]
pub struct MockHoistDependency {
  #[cacheable(with=Skip)]
  call_expr_span: Span,
  #[cacheable(with=Skip)]
  callee_span: Span,
  request: String,
}

impl MockHoistDependency {
  pub fn new(call_expr_span: Span, callee_span: Span, request: String) -> Self {
    Self {
      call_expr_span,
      callee_span,
      request,
    }
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
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let TemplateContext { init_fragments, .. } = code_generatable_context;

    let dep = dep
      .as_any()
      .downcast_ref::<MockHoistDependency>()
      .expect("ModulePathNameDependencyTemplate can only be applied to ModulePathNameDependency");
    let request = &dep.request;

    // Placeholder of hoist target.
    let init = NormalInitFragment::new(
      format!("/* RSTEST:MOCK_PLACEHOLDER:{request} */;"),
      InitFragmentStage::StageConstants,
      0,
      InitFragmentKey::Const(format!("retest mock_hoist {request}")),
      None,
    );

    // Start before hoist.
    let callee_range: DependencyRange = dep.callee_span.into();
    source.replace(
      callee_range.start,
      callee_range.end,
      format!("/* RSTEST:MOCK_HOIST_START:{request} */__webpack_require__.set_mock").as_ref(),
      None,
    );

    // End before hoist.
    let range: DependencyRange = dep.call_expr_span.into();
    source.replace(
      range.end, // count the trailing semicolon
      range.end,
      format!("/* RSTEST:MOCK_HOIST_END:{request} */").as_ref(),
      None,
    );

    init_fragments.push(init.boxed());
  }
}
