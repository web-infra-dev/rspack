use rspack_cacheable::{cacheable, cacheable_dyn, with::Skip};
use rspack_core::{
  AsContextDependency, AsModuleDependency, ConditionalInitFragment, DependencyCodeGeneration,
  DependencyId, DependencyRange, DependencyTemplate, DependencyTemplateType, DependencyType,
  InitFragmentExt, InitFragmentKey, InitFragmentStage, NormalInitFragment, RuntimeCondition,
  TemplateContext, TemplateReplaceSource, import_statement,
};
use swc_core::common::Span;

#[cacheable]
#[derive(Debug, Clone)]
pub enum Position {
  Before,
  After,
}

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
  module_dep_id: Option<DependencyId>,
  position: Position,
}

#[cacheable]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MockMethod {
  Mock,
  DoMock,
  Unmock,
  Hoisted,
}

impl MockMethodDependency {
  pub fn new(
    call_expr_span: Span,
    callee_span: Span,
    request: String,
    hoist: bool,
    method: MockMethod,
    module_dep_id: Option<DependencyId>,
    position: Position,
  ) -> Self {
    Self {
      call_expr_span,
      callee_span,
      request,
      hoist,
      method,
      module_dep_id,
      position,
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
    let TemplateContext {
      module,
      runtime_requirements,
      compilation,
      init_fragments,
      ..
    } = code_generatable_context;
    let dep = dep
      .as_any()
      .downcast_ref::<MockMethodDependency>()
      .expect("ModulePathNameDependencyTemplate can only be applied to ModulePathNameDependency");
    let request = &dep.request;

    let hoist_flag = match dep.method {
      MockMethod::Mock => "MOCK",
      MockMethod::DoMock => "", // won't be used.
      MockMethod::Unmock => "UNMOCK",
      MockMethod::Hoisted => "HOISTED",
    };

    let mock_method = match dep.method {
      MockMethod::Mock => "rstest_mock",
      MockMethod::DoMock => "rstest_do_mock",
      MockMethod::Unmock => "rstest_unmock",
      MockMethod::Hoisted => "rstest_hoisted",
    };

    // Hoist placeholder init fragment.
    let init = NormalInitFragment::new(
      format!("/* RSTEST:{hoist_flag}_PLACEHOLDER:{request} */;"),
      InitFragmentStage::StageESMImports,
      match dep.position {
        Position::Before => 0,
        Position::After => i32::MAX - 1,
      },
      InitFragmentKey::Const(format!("rstest mock_hoist {request}")),
      None,
    );
    init_fragments.push(init.boxed());

    if dep.method == MockMethod::Mock
      && let Some(module_dep_id) = dep.module_dep_id
    {
      let content: (String, String) = import_statement(
        *module,
        compilation,
        runtime_requirements,
        &module_dep_id,
        request,
        false,
      );

      // Redeclaration init fragment.
      init_fragments.push(Box::new(ConditionalInitFragment::new(
        format!("{}{}", content.0, content.1),
        InitFragmentStage::StageAsyncESMImports,
        i32::MAX,
        InitFragmentKey::ESMImport(format!("{} {}", request, "mock")),
        None,
        RuntimeCondition::Boolean(true),
      )));
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
        format!("\n/* RSTEST:{hoist_flag}_HOIST_END:{request} */")
      } else {
        "".to_string()
      }
      .as_ref(),
      None,
    );
  }
}
