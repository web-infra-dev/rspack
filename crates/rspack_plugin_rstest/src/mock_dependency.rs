use rspack_cacheable::{cacheable, cacheable_dyn, with::Skip};
use rspack_core::{
  AsContextDependency, AsModuleDependency, DependencyCodeGeneration, DependencyRange,
  DependencyTemplate, DependencyTemplateType, DependencyType, InitFragmentExt, InitFragmentKey,
  InitFragmentStage, NormalInitFragment, TemplateContext, TemplateReplaceSource,
};
use swc_core::common::Span;

#[cacheable]
#[derive(Debug, Clone)]
pub struct MockDependency {
  #[cacheable(with=Skip)]
  call_expr_span: Span,
  #[cacheable(with=Skip)]
  callee_span: Span,
  request: String,
  hoist: bool,
}

impl MockDependency {
  pub fn new(call_expr_span: Span, callee_span: Span, request: String, hoist: bool) -> Self {
    Self {
      call_expr_span,
      callee_span,
      request,
      hoist,
    }
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for MockDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(MockDependencyTemplate::template_type())
  }
}

impl AsModuleDependency for MockDependency {}
impl AsContextDependency for MockDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct MockDependencyTemplate;

impl MockDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::RstestHoistMock)
  }
}

impl DependencyTemplate for MockDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let TemplateContext { init_fragments, .. } = code_generatable_context;

    let dep = dep
      .as_any()
      .downcast_ref::<MockDependency>()
      .expect("ModulePathNameDependencyTemplate can only be applied to ModulePathNameDependency");
    let request = &dep.request;

    if dep.hoist {
      // Placeholder of hoist target.
      let init = NormalInitFragment::new(
        format!("/* RSTEST:MOCK_PLACEHOLDER:{request} */;"),
        InitFragmentStage::StageConstants,
        0,
        InitFragmentKey::Const(format!("retest mock_hoist {request}")),
        None,
      );
      init_fragments.push(init.boxed());
    }

    // Start before hoist.
    let callee_range: DependencyRange = dep.callee_span.into();
    source.replace(callee_range.start, callee_range.start, "/* ", None);

    source.replace(
      callee_range.end,
      callee_range.end,
      // callee_range.end,
      if dep.hoist {
        format!(" */ /* RSTEST:MOCK_HOIST_START:{request} */__webpack_require__.set_mock")
      } else {
        " */ __webpack_require__.set_mock".to_string()
      }
      .as_str(),
      None,
    );

    // End before hoist.
    let range: DependencyRange = dep.call_expr_span.into();
    source.replace(
      range.end, // count the trailing semicolon
      range.end,
      if dep.hoist {
        format!("/* RSTEST:MOCK_HOIST_END:{request} */")
      } else {
        "".to_string()
      }
      .as_ref(),
      None,
    );
  }
}
