use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsModuleDependency, ConditionalInitFragment, DependencyCodeGeneration,
  DependencyRange, DependencyTemplate, DependencyTemplateType, DependencyType, InitFragmentExt,
  InitFragmentKey, InitFragmentStage, NormalInitFragment, RuntimeCondition, RuntimeGlobals,
  TemplateContext, TemplateReplaceSource,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct MockMethodDependency {
  call_expr_range: DependencyRange,
  callee_range: DependencyRange,
  // Intentionally stored as `DependencyRange` so hoist insertion positions
  // remain cacheable and survive persistent cache restore.
  statement_range: Option<DependencyRange>,
  request: String,
  hoist: bool,
  method: MockMethod,
}

#[cacheable]
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum MockMethod {
  Mock,
  DoMock,
  MockRequire,
  DoMockRequire,
  Unmock,
  DoUnmock,
  Hoisted,
}

impl MockMethodDependency {
  pub fn new(
    call_expr_range: DependencyRange,
    callee_range: DependencyRange,
    request: String,
    hoist: bool,
    method: MockMethod,
  ) -> Self {
    Self {
      call_expr_range,
      callee_range,
      statement_range: None,
      request,
      hoist,
      method,
    }
  }

  pub fn new_with_statement_range(
    call_expr_range: DependencyRange,
    callee_range: DependencyRange,
    statement_range: DependencyRange,
    request: String,
    hoist: bool,
    method: MockMethod,
  ) -> Self {
    Self {
      call_expr_range,
      callee_range,
      statement_range: Some(statement_range),
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
    let TemplateContext {
      init_fragments,
      runtime_template,
      ..
    } = code_generatable_context;
    let dep = dep
      .as_any()
      .downcast_ref::<MockMethodDependency>()
      .expect("MockMethodDependencyTemplate can only be applied to MockMethodDependency");

    let request = &dep.request;
    let require_name = runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE);
    let hoist_id = dep.hoist_id();

    let hoist_flag = Self::get_hoist_flag(&dep.method);
    let mock_method = Self::get_mock_method(&dep.method);

    // Step 1: Add placeholder init fragment for hoistable methods
    if let Some(flag) = hoist_flag {
      Self::add_placeholder_fragment(init_fragments, flag, &hoist_id, request);
    }

    // Step 2: Hoist @rstest/core import to ensure it comes before all hoisted code
    Self::hoist_rstest_core_import(init_fragments);

    // Step 3: Transform the source code
    Self::transform_source(
      source,
      dep,
      &require_name,
      mock_method,
      hoist_flag,
      &hoist_id,
      request,
    );
  }
}

impl MockMethodDependency {
  fn hoist_id(&self) -> String {
    format!(
      "{}-{}",
      self.call_expr_range.start, self.call_expr_range.end
    )
  }
}

impl MockMethodDependencyTemplate {
  /// Get the hoist flag string for methods that need hoisting
  fn get_hoist_flag(method: &MockMethod) -> Option<&'static str> {
    match method {
      MockMethod::Mock => Some("MOCK"),
      MockMethod::MockRequire => Some("MOCKREQUIRE"),
      MockMethod::Unmock => Some("UNMOCK"),
      MockMethod::Hoisted => Some("HOISTED"),
      MockMethod::DoMock | MockMethod::DoMockRequire | MockMethod::DoUnmock => None,
    }
  }

  /// Get the runtime method name
  fn get_mock_method(method: &MockMethod) -> &'static str {
    match method {
      MockMethod::Mock => "rstest_mock",
      MockMethod::DoMock => "rstest_do_mock",
      MockMethod::MockRequire => "rstest_mock_require",
      MockMethod::DoMockRequire => "rstest_do_mock_require",
      MockMethod::Unmock => "rstest_unmock",
      MockMethod::Hoisted => "rstest_hoisted",
      MockMethod::DoUnmock => "rstest_do_unmock",
    }
  }

  /// Add a placeholder init fragment that marks where hoisted code should be inserted
  fn add_placeholder_fragment(
    init_fragments: &mut Vec<Box<dyn rspack_core::InitFragment<rspack_core::GenerateContext<'_>>>>,
    flag: &str,
    hoist_id: &str,
    request: &str,
  ) {
    let init = NormalInitFragment::new(
      format!("/* RSTEST:{flag}:{hoist_id}:{request}:PLACEHOLDER */;"),
      InitFragmentStage::StageESMImports,
      0,
      InitFragmentKey::Const(format!("rstest mock_hoist {hoist_id}")),
      None,
    );
    init_fragments.push(init.boxed());
  }

  /// Hoist @rstest/core import to the very top of the module.
  ///
  /// This ensures that `@rstest/core` is imported before the hoisted placeholder,
  /// so that `rs.fn()` and other utilities are available inside `rs.hoisted()` callbacks.
  ///
  /// We achieve this by inserting a higher-priority fragment with the same key.
  /// Since ESMImport's merge logic returns the first fragment when its runtime_condition is true,
  /// our new fragment will take precedence and the original will be ignored.
  fn hoist_rstest_core_import(
    init_fragments: &mut Vec<Box<dyn rspack_core::InitFragment<rspack_core::GenerateContext<'_>>>>,
  ) {
    let target_key =
      InitFragmentKey::ESMImport("ESM import external global \"@rstest/core\"".to_string());

    // Find the original @rstest/core import fragment
    let Some(fragment) = init_fragments.iter().find(|f| f.key() == &target_key) else {
      return;
    };

    // Clone and downcast to get the content
    let cloned: Box<dyn rspack_core::InitFragment<_>> = fragment.clone();
    let Ok(conditional_fragment) = cloned.into_any().downcast::<ConditionalInitFragment>() else {
      return;
    };

    // Create a new fragment with higher priority (position=-1) and insert at the beginning
    let content = conditional_fragment.content().to_string();
    let rstest_import = ConditionalInitFragment::new(
      content,
      InitFragmentStage::StageESMImports,
      -1, // Higher priority than default (0)
      target_key,
      None,
      RuntimeCondition::Boolean(true),
    );
    init_fragments.insert(0, rstest_import.boxed());
  }

  /// Transform the source code by:
  /// 1. Adding hoist markers (HOIST_START/HOIST_END) around the code to be hoisted
  /// 2. Replacing the original callee with the runtime method
  fn transform_source(
    source: &mut TemplateReplaceSource,
    dep: &MockMethodDependency,
    require_name: &str,
    mock_method: &str,
    hoist_flag: Option<&str>,
    hoist_id: &str,
    request: &str,
  ) {
    let should_hoist = hoist_flag.is_some() && dep.hoist;
    let hoist_marker = hoist_flag.map(|flag| format!("{flag}:{hoist_id}:{request}"));

    if should_hoist && dep.statement_range.is_some() {
      // Case 1: Variable declaration with hoisting (e.g., `const mocks = rs.hoisted(...)`)
      // Wrap the entire statement with hoist markers
      Self::transform_with_statement_hoist(
        source,
        dep,
        require_name,
        mock_method,
        hoist_marker
          .as_deref()
          .expect("hoist marker should exist when should_hoist is true"),
        &dep.callee_range,
      );
    } else if should_hoist {
      // Case 2: Standalone call with hoisting (e.g., `rs.hoisted(...)` or `rs.mock(...)`)
      // Wrap just the call expression with hoist markers
      Self::transform_with_call_hoist(
        source,
        dep,
        require_name,
        mock_method,
        hoist_marker
          .as_deref()
          .expect("hoist marker should exist when should_hoist is true"),
        &dep.callee_range,
      );
    } else {
      // Case 3: No hoisting needed (e.g., `rs.doMock(...)`)
      // Just replace the callee
      Self::transform_without_hoist(source, require_name, mock_method, &dep.callee_range);
    }
  }

  /// Transform for variable declarations that need hoisting.
  /// Example: `const mocks = rs.hoisted(() => {...})`
  /// Result: `/* HOIST_START */const mocks = __webpack_require__.rstest_hoisted(() => {...})/* HOIST_END */`
  fn transform_with_statement_hoist(
    source: &mut TemplateReplaceSource,
    dep: &MockMethodDependency,
    require_name: &str,
    mock_method: &str,
    hoist_marker: &str,
    callee_range: &DependencyRange,
  ) {
    let stmt_range = dep
      .statement_range
      .expect("statement_range should be Some when transform_with_statement_hoist is called");

    // Insert HOIST_START before the statement
    source.replace(
      stmt_range.start,
      stmt_range.start,
      format!("/* RSTEST:{hoist_marker}:HOIST_START */"),
      None,
    );

    // Insert HOIST_END after the statement
    source.replace(
      stmt_range.end,
      stmt_range.end,
      format!("\n/* RSTEST:{hoist_marker}:HOIST_END */"),
      None,
    );

    // Comment out original callee and replace with runtime method
    // `rs.hoisted` -> `/* rs.hoisted */ __webpack_require__.rstest_hoisted`
    source.replace_static(callee_range.start, callee_range.start, "/* ", None);
    source.replace(
      callee_range.end,
      callee_range.end,
      format!(" */ {require_name}.{mock_method}"),
      None,
    );
  }

  /// Transform for standalone calls that need hoisting.
  /// Example: `rs.mock('./foo', () => {...})`
  /// Result: `/* rs.mock */ /* HOIST_START */__webpack_require__.rstest_mock('./foo', () => {...})/* HOIST_END */`
  fn transform_with_call_hoist(
    source: &mut TemplateReplaceSource,
    dep: &MockMethodDependency,
    require_name: &str,
    mock_method: &str,
    hoist_marker: &str,
    callee_range: &DependencyRange,
  ) {
    // Comment out original callee and add HOIST_START + runtime method
    source.replace_static(callee_range.start, callee_range.start, "/* ", None);
    source.replace(
      callee_range.end,
      callee_range.end,
      format!(" */ /* RSTEST:{hoist_marker}:HOIST_START */{require_name}.{mock_method}"),
      None,
    );

    // Insert HOIST_END after the call expression
    source.replace(
      dep.call_expr_range.end,
      dep.call_expr_range.end,
      format!("\n/* RSTEST:{hoist_marker}:HOIST_END */"),
      None,
    );
  }

  /// Transform for calls without hoisting.
  /// Example: `rs.doMock('./foo', () => {...})`
  /// Result: `/* rs.doMock */ __webpack_require__.rstest_do_mock('./foo', () => {...})`
  fn transform_without_hoist(
    source: &mut TemplateReplaceSource,
    require_name: &str,
    mock_method: &str,
    callee_range: &DependencyRange,
  ) {
    source.replace_static(callee_range.start, callee_range.start, "/* ", None);
    source.replace(
      callee_range.end,
      callee_range.end,
      format!(" */ {require_name}.{mock_method}"),
      None,
    );
  }
}
