use rspack_cacheable::{cacheable, cacheable_dyn, with::Skip};
use rspack_core::{
  AsContextDependency, AsModuleDependency, DependencyCodeGeneration, DependencyRange,
  DependencyTemplate, DependencyTemplateType, DependencyType, InitFragmentExt, InitFragmentKey,
  InitFragmentStage, NormalInitFragment, TemplateContext, TemplateReplaceSource,
};
use swc_core::common::Span;

#[cacheable]
#[derive(Debug, Clone)]
pub struct MockMethodDependency {
  #[cacheable(with=Skip)]
  call_expr_span: Span,
  #[cacheable(with=Skip)]
  callee_span: Span,
  request: String,
  hoist: bool,
  method: MockMethod,
}

#[cacheable]
#[derive(Debug, Clone)]
pub enum MockMethod {
  Mock,
  Unmock,
}

impl MockMethodDependency {
  pub fn new(
    call_expr_span: Span,
    callee_span: Span,
    request: String,
    hoist: bool,
    method: MockMethod,
  ) -> Self {
    Self {
      call_expr_span,
      callee_span,
      request,
      hoist,
      method,
    }
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for MockMethodDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(MockMethodDependencyTemplate::template_type())
  }
}

impl AsModuleDependency for MockMethodDependency {}
impl AsContextDependency for MockMethodDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct MockMethodDependencyTemplate;

impl MockMethodDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::RstestHoistMock)
  }
}

impl DependencyTemplate for MockMethodDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let TemplateContext { init_fragments, .. } = code_generatable_context;

    let dep = dep
      .as_any()
      .downcast_ref::<MockMethodDependency>()
      .expect("ModulePathNameDependencyTemplate can only be applied to ModulePathNameDependency");
    let request = &dep.request;

    let hoist_flag = match dep.method {
      MockMethod::Mock => "MOCK",
      MockMethod::Unmock => "UNMOCK",
    };

    let mock_method = match dep.method {
      MockMethod::Mock => "set_mock",
      MockMethod::Unmock => "unmock",
    };

    if dep.hoist {
      // Placeholder of hoist target.
      let init = NormalInitFragment::new(
        format!("/* RSTEST:{hoist_flag}_PLACEHOLDER:{request} */;"),
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
        format!(
          " */ /* RSTEST:{hoist_flag}_HOIST_START:{request} */__webpack_require__.{mock_method}"
        )
      } else {
        format!(" */ __webpack_require__.{mock_method}")
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
        format!("/* RSTEST:{hoist_flag}_HOIST_END:{request} */")
      } else {
        "".to_string()
      }
      .as_ref(),
      None,
    );
  }
}
