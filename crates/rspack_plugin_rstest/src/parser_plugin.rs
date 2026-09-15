use std::sync::Arc;

use camino::Utf8PathBuf;
use rspack_core::{
  AsyncDependenciesBlock, BoxDependency, ConstDependency, DependencyRange, ImportAttributes,
  ImportPhase,
};
use rspack_intern::Atom;
use rspack_plugin_javascript::{
  JavascriptParserPlugin,
  dependency::{CommonJsRequireDependency, ImportDependency, RequireHeaderDependency},
  try_extract_magic_comment,
  utils::{
    self,
    eval::{self},
  },
  visitors::{JavascriptParser, Statement, VariableDeclaration, create_traceable_error, expr_name},
};
use rspack_util::{SpanExt, json_stringify_str, swc::get_swc_comments};
use swc_experimental_ecma_ast::{
  CallExpr, Callee, GetSpan, Ident, IdentName, ImportDecl, ImportPhase as AstImportPhase,
  MemberExpr, MetaPropKind, OptChainBase, OptChainExpr, Span, UnaryExpr, VarDeclarator,
};

static RSTEST_MOCK_FIRST_ARG_TAG: &str = "strip the import call from the first arg of mock series";
static RSTEST_API_IMPORT_TAG: &str = "rstest test api import";

use crate::{
  dynamic_import_origin_dependency::RstestDynamicImportOriginDependency,
  mock_method_dependency::{MockMethod, MockMethodDependency},
  mock_module_id_dependency::MockModuleIdDependency,
  module_path_name_dependency::{ModulePathNameDependency, NameType},
  require_resolve_origin_dependency::RstestRequireResolveOriginDependency,
};

const DIR_NAME: &str = "__dirname";
const FILE_NAME: &str = "__filename";
const IMPORT_META_DIRNAME: &str = "import.meta.dirname";
const IMPORT_META_FILENAME: &str = "import.meta.filename";
const IMPORT_META_RSTEST: &str = "import.meta.rstest";
pub(crate) const MOCK_TARGET_REQUEST_PREFIX: &str = "\0rstest_mock_target:\0";

#[derive(PartialEq)]
enum ModulePathType {
  DirName,
  FileName,
}

#[derive(Debug, Clone)]
pub struct RstestParserPluginOptions {
  pub module_path_name: bool,
  pub hoist_mock_module: bool,
  pub import_meta_path_name: bool,
  pub manual_mock_root: String,
  /// Whether to handle global `rs` and `rstest` variables.
  /// When false, only ESM imported variables are processed.
  pub globals: bool,
  /// Whether to replace `import.meta.rstest` with the optional runtime resolver
  /// call carrying the source module's absolute path.
  pub inject_import_meta_rstest_origin: bool,
  /// Whether to rewrite non-string-literal `import()` calls with origin info.
  /// Pre-resolved at plugin construction — false here covers both "feature
  /// disabled" and "callee resolved to default `import`".
  pub inject_dynamic_import_origin: bool,
  /// Whether to rewrite `require.resolve()` calls with origin info.
  pub inject_require_resolve_origin: bool,
  /// Whether to respect `/* webpackIgnore: true */` in CommonJS calls.
  pub commonjs_magic_comments: bool,
}

impl Default for RstestParserPluginOptions {
  fn default() -> Self {
    Self {
      module_path_name: false,
      hoist_mock_module: false,
      import_meta_path_name: false,
      manual_mock_root: String::new(),
      globals: true,
      inject_import_meta_rstest_origin: false,
      inject_dynamic_import_origin: false,
      inject_require_resolve_origin: false,
      commonjs_magic_comments: false,
    }
  }
}

#[derive(Debug, Default)]
pub struct RstestParserPlugin {
  options: RstestParserPluginOptions,
}

impl RstestParserPlugin {
  pub fn new(options: RstestParserPluginOptions) -> Self {
    Self { options }
  }

  fn has_ignore_comment(
    &self,
    parser: &mut JavascriptParser,
    error_span: Span,
    span: Span,
  ) -> bool {
    if !self.options.commonjs_magic_comments {
      return false;
    }

    try_extract_magic_comment(parser, error_span, span)
      .get_ignore()
      .unwrap_or_default()
  }

  fn process_require_resolve_origin(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
  ) -> Option<bool> {
    let callee_expr = call_expr.callee.as_expr()?;
    let member_expr = callee_expr.as_member()?;
    let require_ident = member_expr.obj.as_ident()?;

    if parser.get_variable_info(&require_ident.sym).is_some() {
      return None;
    }

    if !(1..=2).contains(&call_expr.args.len()) {
      return None;
    }

    let first_arg = call_expr.args.first()?;
    if first_arg.spread.is_some()
      || self.has_ignore_comment(parser, call_expr.span, first_arg.span())
    {
      return None;
    }

    if call_expr
      .args
      .get(1)
      .is_some_and(|arg| arg.spread.is_some())
    {
      return None;
    }

    let resource_path = parser.resource_data.path()?;
    let origin_path = resource_path.as_str().to_string();

    let last_arg = call_expr
      .args
      .last()
      .expect("call_expr.args has at least one element");
    parser.add_presentational_dependency(Arc::new(RstestRequireResolveOriginDependency::new(
      call_expr.callee.span().into(),
      last_arg.span().real_hi(),
      origin_path,
    )));

    // Returning `Some(true)` short-circuits the default walker for this call,
    // so preserve dependency collection for nested expressions in arguments.
    parser.walk_expr_or_spread(&call_expr.args);
    Some(true)
  }

  fn import_meta_rstest_expression(&self, parser: &JavascriptParser) -> Option<String> {
    if !self.options.inject_import_meta_rstest_origin {
      return None;
    }
    let resource_path = parser.resource_data.path()?;
    Some(format!(
      "globalThis['@rstest/core/import-meta']?.({})",
      json_stringify_str(resource_path.as_str())
    ))
  }

  fn process_require_actual(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
  ) -> Option<bool> {
    match call_expr.args.len() {
      1 => {
        let first_arg = &call_expr.args[0];
        if let Some(lit) = first_arg.expr.as_lit()
          && let Some(lit) = lit.as_str()
        {
          let first_arg_range: DependencyRange = first_arg.span().into();
          let loc = parser.to_dependency_location(first_arg_range);
          let dep = CommonJsRequireDependency::new(
            lit.value.to_string_lossy().to_string(),
            first_arg_range,
            Some(call_expr.span.into()),
            parser.in_try,
            loc,
          );
          parser.add_dependency(BoxDependency::new(dep));

          let callee_range = call_expr.callee.span().into();
          let loc = parser.to_dependency_location(callee_range);
          parser.add_presentational_dependency(Arc::new(RequireHeaderDependency::new(
            callee_range,
            loc,
          )));

          parser.add_presentational_dependency(Arc::new(ConstDependency::new(
            callee_range,
            ".rstest_require_actual".into(),
          )));

          return Some(true);
        }
      }
      _ => {
        parser.add_error(
          create_traceable_error(
            "Invalid function call".into(),
            "`rs.requireActual` function expects 1 argument".into(),
            parser.source().to_string(),
            call_expr.span.into(),
          )
          .into(),
        );
      }
    }

    None
  }

  fn process_import_actual(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
  ) -> Option<bool> {
    match call_expr.args.len() {
      1 => {
        let first_arg = &call_expr.args[0];
        if let Some(lit) = first_arg.expr.as_lit()
          && let Some(lit) = lit.as_str()
        {
          let mut attrs = ImportAttributes::default();
          attrs.insert("rstest".to_string(), "importActual".to_string());

          let imported_span = call_expr.args.first().expect("should have one arg");

          let range = call_expr.span.into();
          let dep = BoxDependency::new(ImportDependency::new(
            Atom::from(lit.value.to_string_lossy().as_ref()),
            range,
            Some(attrs),
            ImportPhase::Evaluation,
            parser.in_try,
            get_swc_comments(
              parser.ast.comments,
              imported_span.span().start,
              imported_span.span().end,
            ),
          ));

          let loc = parser.to_dependency_location(range);
          let block = AsyncDependenciesBlock::new(
            *parser.module_identifier,
            loc,
            None,
            vec![dep],
            Some(lit.value.to_string_lossy().to_string()),
          );

          parser.add_block(Box::new(block));
          return Some(true);
        }
      }
      _ => {
        parser.add_error(
          create_traceable_error(
            "Invalid function call".into(),
            "`rs.importActual` function expects 1 argument".into(),
            parser.source().to_string(),
            call_expr.span.into(),
          )
          .into(),
        );
      }
    }

    None
  }

  fn calc_mocked_target(&self, value: &str) -> Utf8PathBuf {
    // node:foo will be mocked to `__mocks__/foo`.
    let stripped = value.strip_prefix("node:").unwrap_or(value);
    let path_buf = Utf8PathBuf::from(stripped);
    let is_relative_request = stripped.starts_with('.'); // TODO: consider alias?

    if is_relative_request {
      // Keep using sibling `__mocks__` by default. Directory-entry mocks are
      // rewritten later in NMF with the actual resolver result.
      path_buf.parent().map_or_else(
        || Utf8PathBuf::from("__mocks__").join(&path_buf),
        |p| {
          p.join("__mocks__")
            .join(path_buf.file_name().unwrap_or_default())
        },
      )
    } else {
      // Mock non-relative request to `manual_mock_root` directory.
      Utf8PathBuf::from(&self.options.manual_mock_root).join(&path_buf)
    }
  }

  fn handle_mock_first_arg(
    &self,
    parser: &mut JavascriptParser,
    mock_call_expr: &CallExpr,
  ) -> Option<String> {
    let first_arg = &mock_call_expr.args[0];
    let mut is_import_call = false;

    if let Some(first_arg) = mock_call_expr.args.first()
      && let Some(import_call) = first_arg.expr.as_call()
      && import_call.callee.as_import().is_some()
    {
      parser.tag_variable::<bool>(
        self.compose_rstest_import_call_key(import_call).into(),
        RSTEST_MOCK_FIRST_ARG_TAG,
        Some(true),
      );
      is_import_call = true;
    }

    let lit_str = if is_import_call {
      first_arg
        .expr
        .as_call()
        .and_then(|expr| expr.args.first())
        .and_then(|arg| arg.expr.as_lit())
        .and_then(|lit| lit.as_str())
        .and_then(|lit| lit.value.as_str())
    } else {
      first_arg
        .expr
        .as_lit()
        .and_then(|lit| lit.as_str())
        .and_then(|lit| lit.value.as_str())
    };

    lit_str.map(|s| s.to_string())
  }

  #[allow(clippy::too_many_arguments)]
  fn process_mock(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
    hoist: bool,
    is_esm: bool,
    method: MockMethod,
    has_b: bool,
    test_api_import_source_order: Option<i32>,
  ) {
    match call_expr.args.len() {
      1 => {
        let first_arg = &call_expr.args[0];
        let first_arg_lit_str = self.handle_mock_first_arg(parser, call_expr);

        if let Some(lit_str) = first_arg_lit_str {
          let dep = MockModuleIdDependency::new(
            lit_str.clone(),
            first_arg.span().into(),
            false,
            true,
            if is_esm {
              rspack_core::DependencyCategory::Esm
            } else {
              rspack_core::DependencyCategory::CommonJS
            },
            if has_b { Some(", ".to_string()) } else { None },
          );
          parser.add_dependency(BoxDependency::new(dep));

          parser.add_presentational_dependency(Arc::new(
            MockMethodDependency::new(
              call_expr.span().into(),
              call_expr.callee.span().into(),
              lit_str.clone(),
              hoist,
              method,
            )
            // has_b=false (1-arg `rs.unmock('X')`): append request after the id.
            // has_b=true (1-arg auto-mock): request rides the synthetic-target
            // suffix below instead — skip here to avoid a same-offset collision.
            .with_request_arg_end(if has_b {
              None
            } else {
              Some(first_arg.span().real_hi())
            })
            .with_test_api_import_source_order(test_api_import_source_order),
          ));

          if has_b {
            let second_arg = Span::new(first_arg.span().end, first_arg.span().end);
            parser.add_dependency(BoxDependency::new(
              MockModuleIdDependency::new(
                format!("{MOCK_TARGET_REQUEST_PREFIX}{lit_str}"),
                second_arg.into(),
                false,
                true,
                if is_esm {
                  rspack_core::DependencyCategory::Esm
                } else {
                  rspack_core::DependencyCategory::CommonJS
                },
                // Render the synthetic target id followed by the clean request
                // literal, yielding `rstest_mock(<id>, <targetId>, "X")`.
                Some(format!(", {}", json_stringify_str(&lit_str))),
              )
              // `rs.mock('X')` first tries to resolve a manual mock target. If no
              // `__mocks__` file exists, fall back to Vitest-style automocking by
              // passing `{ mock: true }` to the runtime, equivalent to
              // `rs.mock('X', { mock: true })`.
              .with_missing_module_fallback("{ mock: true }".to_string()),
            ));
          }
        }
      }
      // mock a module
      2 => {
        let first_arg = &call_expr.args[0];
        let second_arg = &call_expr.args[1];

        if first_arg.spread.is_some() || second_arg.spread.is_some() {
          return;
        }

        let lit_str = self.handle_mock_first_arg(parser, call_expr);

        if let Some(lit_str) = lit_str {
          let module_dep = MockModuleIdDependency::new(
            lit_str.clone(),
            first_arg.span().into(),
            false,
            true,
            if is_esm {
              rspack_core::DependencyCategory::Esm
            } else {
              rspack_core::DependencyCategory::CommonJS
            },
            None,
          );

          parser.add_presentational_dependency(Arc::new(
            MockMethodDependency::new(
              call_expr.span().into(),
              call_expr.callee.span().into(),
              lit_str,
              hoist,
              method,
            )
            // 2-arg `rs.mock('X', factory)`: append request after the factory.
            .with_request_arg_end(Some(second_arg.span().real_hi()))
            .with_test_api_import_source_order(test_api_import_source_order),
          ));
          parser.add_dependency(BoxDependency::new(module_dep));
        } else {
          parser.add_error(
            create_traceable_error(
              "Invalid function call".into(),
              "`rs.mock` function expects a string literal as the first argument".into(),
              parser.source().to_string(),
              call_expr.span.into(),
            )
            .into(),
          );
        }
      }
      _ => {
        parser.add_error(
          create_traceable_error(
            "Invalid function call".into(),
            "`rs.mock` function expects 1 or 2 arguments".into(),
            parser.source().to_string(),
            call_expr.span.into(),
          )
          .into(),
        );
      }
    }
  }

  fn hoisted(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
    statement_span: Option<Span>,
    test_api_import_source_order: Option<i32>,
  ) -> Option<bool> {
    match call_expr.args.len() {
      1 => {
        let dep = if let Some(stmt_span) = statement_span {
          MockMethodDependency::new_with_statement_range(
            call_expr.span().into(),
            call_expr.callee.span().into(),
            stmt_span.into(),
            call_expr.span().real_lo().to_string(),
            true,
            MockMethod::Hoisted,
          )
        } else {
          MockMethodDependency::new(
            call_expr.span().into(),
            call_expr.callee.span().into(),
            call_expr.span().real_lo().to_string(),
            true,
            MockMethod::Hoisted,
          )
        };
        parser.add_presentational_dependency(Arc::new(
          dep.with_test_api_import_source_order(test_api_import_source_order),
        ));
        Some(false)
      }
      _ => {
        parser.add_error(
          create_traceable_error(
            "Invalid function call".into(),
            "`rs.hoisted` function expects 1 argument".into(),
            parser.source().to_string(),
            call_expr.span.into(),
          )
          .into(),
        );
        Some(false)
      }
    }
  }

  fn reset_modules(&self, parser: &mut JavascriptParser, call_expr: &CallExpr) -> Option<bool> {
    match call_expr.args.len() {
      0 => {
        parser.add_presentational_dependency(Arc::new(ConstDependency::new(
          call_expr.callee.span().into(),
          format!(
            "{}.rstest_reset_modules",
            parser.parser_runtime_requirements.require
          )
          .into(),
        )));
        Some(true)
      }
      _ => {
        parser.add_error(
          create_traceable_error(
            "Invalid function call".into(),
            "`rs.resetModules` function expects 0 arguments".into(),
            parser.source().to_string(),
            call_expr.span.into(),
          )
          .into(),
        );
        Some(false)
      }
    }
  }

  fn load_mock(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
    is_esm: bool,
  ) -> Option<bool> {
    match call_expr.args.len() {
      1 => {
        let first_arg = &call_expr.args[0];
        if let Some(lit) = first_arg.expr.as_lit() {
          if let Some(lit) = lit.as_str() {
            if let Some(mocked_target) = self
              .calc_mocked_target(&lit.value.to_string_lossy())
              .as_std_path()
              .to_str()
            {
              if is_esm {
                let imported_span = call_expr.args.first().expect("should have one arg");

                let mut attrs = ImportAttributes::default();
                attrs.insert("rstest".to_string(), "importMock".to_string());
                let range = call_expr.span.into();
                let dep = BoxDependency::new(ImportDependency::new(
                  Atom::from(mocked_target),
                  range,
                  Some(attrs),
                  ImportPhase::Evaluation,
                  parser.in_try,
                  get_swc_comments(
                    parser.ast.comments,
                    imported_span.span().start,
                    imported_span.span().end,
                  ),
                ));

                let loc = parser.to_dependency_location(range);
                let block = AsyncDependenciesBlock::new(
                  *parser.module_identifier,
                  loc,
                  None,
                  vec![dep],
                  Some(mocked_target.to_string()),
                );

                parser.add_block(Box::new(block));

                return Some(true);
              } else {
                let first_arg_range = first_arg.span().into();
                let loc = parser.to_dependency_location(first_arg_range);
                let dep: CommonJsRequireDependency = CommonJsRequireDependency::new(
                  mocked_target.to_string(),
                  first_arg_range,
                  Some(call_expr.span.into()),
                  parser.in_try,
                  loc,
                );

                let callee_range = call_expr.callee.span().into();
                let loc = parser.to_dependency_location(callee_range);
                parser.add_presentational_dependency(Arc::new(RequireHeaderDependency::new(
                  callee_range,
                  loc,
                )));

                parser.add_dependency(BoxDependency::new(dep));
                return Some(true);
              }
            }
          } else {
            return None;
          }
        }

        None
      }
      _ => {
        parser.add_error(
          create_traceable_error(
            "Invalid function call".into(),
            "`rs.importMock` or `rs.requireMock` function expects 1 argument".into(),
            parser.source().to_string(),
            call_expr.span.into(),
          )
          .into(),
        );
        Some(false)
      }
    }
  }

  fn process_import_meta(&self, parser: &mut JavascriptParser, r#type: ModulePathType) -> String {
    if r#type == ModulePathType::FileName {
      if let Some(resource_path) = parser.resource_data.path() {
        json_stringify_str(resource_path.as_str())
      } else {
        "''".to_string()
      }
    } else {
      let resource_path = parser
        .resource_data
        .path()
        .and_then(|p| p.parent())
        .map(|p| p.to_string())
        .unwrap_or_default();
      json_stringify_str(&resource_path)
    }
  }

  fn compose_rstest_import_call_key(&self, call_expr: &CallExpr) -> String {
    format!(
      "rstest_strip_import_call {} {}",
      call_expr.span.real_lo(),
      call_expr.span.real_hi(),
    )
  }

  fn handle_rstest_method_call(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
    ident: &Ident,
    prop: &IdentName,
    statement_span: Option<Span>,
  ) -> Option<bool> {
    let test_api_import_source_order = parser
      .get_tag_data::<i32>(&ident.sym, RSTEST_API_IMPORT_TAG)
      .copied();

    // Check if this is a global variable (free variable) or an ESM import
    let is_global = !parser.is_variable_defined(&ident.sym);

    // Skip global variables if globals option is disabled
    if is_global && !self.options.globals {
      return None;
    }

    match (ident.sym.as_str(), prop.sym.as_str()) {
      // rs.mock
      ("rs" | "rstest", "mock") => {
        self.process_mock(
          parser,
          call_expr,
          true,
          true,
          MockMethod::Mock,
          true,
          test_api_import_source_order,
        );
        Some(false)
      }
      // rs.mockRequire
      ("rs" | "rstest", "mockRequire") => {
        self.process_mock(
          parser,
          call_expr,
          true,
          false,
          MockMethod::MockRequire,
          true,
          test_api_import_source_order,
        );
        Some(false)
      }
      // rs.doMock
      ("rs" | "rstest", "doMock") => {
        self.process_mock(
          parser,
          call_expr,
          false,
          true,
          MockMethod::DoMock,
          true,
          test_api_import_source_order,
        );
        Some(false)
      }
      // rs.doMockRequire
      ("rs" | "rstest", "doMockRequire") => {
        self.process_mock(
          parser,
          call_expr,
          false,
          false,
          MockMethod::DoMockRequire,
          true,
          test_api_import_source_order,
        );
        Some(false)
      }
      // rs.importActual and rs.requireActual are handled by call_member_chain hook
      // rs.importMock
      ("rs" | "rstest", "importMock") => self.load_mock(parser, call_expr, true),
      // rs.requireMock
      ("rs" | "rstest", "requireMock") => self.load_mock(parser, call_expr, false),
      // rs.unmock
      ("rs" | "rstest", "unmock") => {
        self.process_mock(
          parser,
          call_expr,
          true,
          true,
          MockMethod::Unmock,
          false,
          test_api_import_source_order,
        );
        Some(true)
      }
      // rs.doUnmock
      ("rs" | "rstest", "doUnmock") => {
        self.process_mock(
          parser,
          call_expr,
          false,
          true,
          MockMethod::Unmock,
          false,
          test_api_import_source_order,
        );
        Some(true)
      }
      // rs.unmockRequire
      ("rs" | "rstest", "unmockRequire") => {
        self.process_mock(
          parser,
          call_expr,
          true,
          false,
          MockMethod::Unmock,
          false,
          test_api_import_source_order,
        );
        Some(true)
      }
      // rs.doUnmockRequire
      ("rs" | "rstest", "doUnmockRequire") => {
        self.process_mock(
          parser,
          call_expr,
          false,
          false,
          MockMethod::Unmock,
          false,
          test_api_import_source_order,
        );
        Some(true)
      }
      // rs.resetModules
      ("rs" | "rstest", "resetModules") => self.reset_modules(parser, call_expr),
      // rs.hoisted
      ("rs" | "rstest", "hoisted") => self.hoisted(
        parser,
        call_expr,
        statement_span,
        test_api_import_source_order,
      ),
      _ => {
        // Not a mock module, continue.
        None
      }
    }
  }
}

#[rspack_plugin_javascript::implemented_javascript_parser_hooks]
impl<'p, 'a> JavascriptParserPlugin<'p, 'a> for RstestParserPlugin {
  fn import_specifier(
    &self,
    parser: &mut JavascriptParser<'p>,
    _statement: &ImportDecl,
    _source: &Atom,
    _export_name: Option<&Atom>,
    identifier_name: &Atom,
  ) -> Option<bool> {
    if matches!(identifier_name.as_str(), "rs" | "rstest") {
      let source_order = parser.current_esm_import_order();
      parser.tag_variable::<i32>(
        identifier_name.clone(),
        RSTEST_API_IMPORT_TAG,
        Some(source_order),
      );
    }

    None
  }

  fn declarator(
    &self,
    parser: &mut JavascriptParser<'p>,
    _expr: &VarDeclarator,
    stmt: VariableDeclaration<'_>,
  ) -> Option<bool> {
    for decl in stmt.declarators() {
      if let Some(init) = &decl.init {
        let call_expr = match init.as_call() {
          Some(call) => Some(call),
          None => init
            .as_await()
            .and_then(|await_expr| await_expr.arg.as_call()),
        };

        if let Some(call_expr) = call_expr
          && let Some(callee_expr) = call_expr.callee.as_expr()
          && let Some(member_expr) = callee_expr.as_member()
          && let Some(obj_ident) = member_expr.obj.as_ident()
          && let Some(prop_ident) = member_expr.prop.as_ident()
        {
          return self.handle_rstest_method_call(
            parser,
            call_expr,
            obj_ident,
            prop_ident,
            Some(stmt.span()),
          );
        }
      }
    }

    None
  }

  fn statement(&self, parser: &mut JavascriptParser<'p>, stmt: Statement) -> Option<bool> {
    let call_expr = match stmt {
      Statement::Expr(expr_stmt) if expr_stmt.expr.as_call().is_some() => expr_stmt
        .expr
        .as_call()
        .expect("call expression should exist after checking with is_some()"),
      _ => return None,
    };

    if !self.options.hoist_mock_module {
      return None;
    }

    if let Some(callee_expr) = call_expr.callee.as_expr()
      && let Some(member_expr) = callee_expr.as_member()
      && let Some(obj_ident) = member_expr.obj.as_ident()
      && let Some(prop_ident) = member_expr.prop.as_ident()
    {
      return self.handle_rstest_method_call(parser, call_expr, obj_ident, prop_ident, None);
    }

    None
  }

  fn call(
    &self,
    parser: &mut JavascriptParser<'p>,
    call_expr: &CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    if self.options.inject_require_resolve_origin && for_name == "require.resolve" {
      return self.process_require_resolve_origin(parser, call_expr);
    }

    None
  }

  fn import_call(
    &self,
    parser: &mut JavascriptParser<'p>,
    call_expr: &CallExpr,
    import_then: Option<&CallExpr>,
    _members: Option<(&[Atom], bool)>,
  ) -> Option<bool> {
    let first_arg = self.handle_mock_first_arg(parser, call_expr);
    if first_arg.is_some() {
      let tag_data = parser.get_tag_data::<bool>(
        &self.compose_rstest_import_call_key(call_expr),
        RSTEST_MOCK_FIRST_ARG_TAG,
      );

      if tag_data.is_some() {
        return Some(true);
      }
    }

    if self.options.inject_dynamic_import_origin {
      // Only handle the regular evaluation phase. `import.defer(...)` and
      // `import.source(...)` carry phase semantics that rstest's runtime
      // does not implement, and the default `ImportParserPlugin` enforces
      // the `experiments.deferImport` gate which we must not bypass.
      let import_node = call_expr.callee.as_import()?;
      if !matches!(import_node.phase, AstImportPhase::Evaluation) {
        return None;
      }

      // Mirror `ImportParserPlugin.import_call`'s `/* webpackIgnore: true */`
      // bailout so authors can opt out of rewriting on a per-call basis.
      let arg = call_expr.args.first()?;
      if arg.spread.is_some() {
        return None;
      }

      let magic = try_extract_magic_comment(parser, call_expr.span, arg.span());
      if magic.get_ignore().unwrap_or_default() {
        return None;
      }

      let param = parser.evaluate_expression(&arg.expr);
      if param.is_string() {
        return None;
      }

      let resource_path = parser.resource_data.path()?;
      let origin_path = resource_path.as_str().to_string();

      let last_arg = call_expr
        .args
        .last()
        .expect("call_expr.args has at least one element");
      let args_end = last_arg.span().real_hi();
      let has_attributes = call_expr.args.len() >= 2;

      parser.add_presentational_dependency(Arc::new(RstestDynamicImportOriginDependency::new(
        call_expr.callee.span().into(),
        args_end,
        has_attributes,
        origin_path,
      )));

      // Returning `Some(true)` short-circuits the parser's walk of this
      // `import()` node (see `walk.rs` `Callee::Import` branch), so we must
      // walk nested expressions ourselves — otherwise `require(...)` or
      // `import()` calls inside the specifier or the `.then` callback get
      // dropped from the dependency graph.
      parser.walk_expr_or_spread(&call_expr.args);
      if let Some(import_then) = import_then {
        parser.walk_expr_or_spread(&import_then.args);
      }

      return Some(true);
    }

    None
  }

  fn call_member_chain(
    &self,
    parser: &mut JavascriptParser<'p>,
    call_expr: &CallExpr,
    for_name: &str,
    members: &[Atom],
    _members_optionals: &[bool],
    _member_ranges: &[Span],
  ) -> Option<bool> {
    // Handle rs.requireActual and rs.importActual calls
    // Extract the variable name from call_expr.callee to handle both:
    // 1. Global variables: rs.importActual() or rstest.importActual()
    // 2. ESM imports: import { rs } from '@rstest/core'; rs.importActual()
    if members.len() == 1
      && let Callee::Expr(callee) = &call_expr.callee
      && let Some(member_expr) = callee.as_member()
      && let Some(ident) = member_expr.obj.as_ident()
    {
      let var_name = ident.sym.as_str();
      if var_name == "rs" || var_name == "rstest" {
        // Check if this is a global variable (for_name matches var_name)
        // or ESM import (for_name is the ESM specifier tag)
        let is_global = for_name == var_name;

        // Skip global variables if globals option is disabled
        if is_global && !self.options.globals {
          return None;
        }
        match members[0].as_str() {
          "requireActual" => {
            return self.process_require_actual(parser, call_expr);
          }
          "importActual" => {
            return self.process_import_actual(parser, call_expr);
          }
          _ => {}
        }
      }
    }
    None
  }

  fn identifier(
    &self,
    parser: &mut JavascriptParser<'p>,
    _ident: &Ident,
    for_name: &str,
  ) -> Option<bool> {
    if self.options.module_path_name {
      match for_name {
        DIR_NAME => {
          parser.add_presentational_dependency(Arc::new(ModulePathNameDependency::new(
            NameType::DirName,
          )));
          return Some(true);
        }
        FILE_NAME => {
          parser.add_presentational_dependency(Arc::new(ModulePathNameDependency::new(
            NameType::FileName,
          )));
          return Some(true);
        }
        _ => return None,
      }
    }

    None
  }

  fn evaluate_typeof(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: &'a UnaryExpr,
    for_name: &str,
  ) -> Option<utils::eval::BasicEvaluatedExpression<'a>> {
    if for_name == IMPORT_META_RSTEST && self.import_meta_rstest_expression(parser).is_some() {
      return Some(eval::BasicEvaluatedExpression::with_range(
        expr.span.real_lo(),
        expr.span.real_hi(),
      ));
    }

    if self.options.import_meta_path_name {
      let mut evaluated = None;
      if for_name == IMPORT_META_DIRNAME || for_name == IMPORT_META_FILENAME {
        evaluated = Some("string".to_string());
      }
      return evaluated
        .map(|e| eval::evaluate_to_string(e, expr.span.real_lo(), expr.span.real_hi()));
    }

    None
  }

  fn evaluate_identifier(
    &self,
    parser: &mut JavascriptParser<'p>,
    for_name: &str,
    _member_expr_info: Option<&rspack_plugin_javascript::visitors::ExpressionExpressionInfo>,
    start: u32,
    end: u32,
  ) -> Option<eval::BasicEvaluatedExpression<'p>> {
    if for_name == IMPORT_META_RSTEST && self.import_meta_rstest_expression(parser).is_some() {
      return Some(eval::BasicEvaluatedExpression::with_range(start, end));
    }

    if self.options.inject_require_resolve_origin && for_name == expr_name::REQUIRE_RESOLVE {
      return Some(eval::evaluate_to_identifier(
        expr_name::REQUIRE_RESOLVE.into(),
        expr_name::REQUIRE_RESOLVE.into(),
        Some(true),
        start,
        end,
      ));
    }

    if self.options.import_meta_path_name {
      if for_name == IMPORT_META_DIRNAME {
        return Some(eval::evaluate_to_string(
          self.process_import_meta(parser, ModulePathType::DirName),
          start,
          end,
        ));
      } else if for_name == IMPORT_META_FILENAME {
        return Some(eval::evaluate_to_string(
          self.process_import_meta(parser, ModulePathType::FileName),
          start,
          end,
        ));
      } else {
        return None;
      }
    }
    None
  }

  fn r#typeof(
    &self,
    parser: &mut JavascriptParser<'p>,
    unary_expr: &UnaryExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == IMPORT_META_RSTEST
      && let Some(expression) = self.import_meta_rstest_expression(parser)
    {
      parser.add_presentational_dependency(Arc::new(ConstDependency::new(
        unary_expr.span().into(),
        format!("typeof ({expression})").into(),
      )));
      return Some(true);
    }

    if self.options.import_meta_path_name {
      if for_name == IMPORT_META_DIRNAME || for_name == IMPORT_META_FILENAME {
        parser.add_presentational_dependency(Arc::new(ConstDependency::new(
          unary_expr.span().into(),
          "'string'".into(),
        )));
        return Some(true);
      } else {
        return None;
      }
    }

    None
  }

  fn member(
    &self,
    parser: &mut JavascriptParser<'p>,
    member_expr: &MemberExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == IMPORT_META_RSTEST
      && let Some(expression) = self.import_meta_rstest_expression(parser)
    {
      // TODO: Replace this Rstest-specific parser rewrite with
      // DefinePlugin.runtimeValue once Rspack supports the webpack API.
      parser.add_presentational_dependency(Arc::new(ConstDependency::new(
        member_expr.span().into(),
        expression.into(),
      )));
      return Some(true);
    }

    if self.options.import_meta_path_name {
      if for_name == IMPORT_META_DIRNAME {
        let result = self.process_import_meta(parser, ModulePathType::DirName);
        parser.add_presentational_dependency(Arc::new(ConstDependency::new(
          member_expr.span().into(),
          result.into(),
        )));
        return Some(true);
      } else if for_name == IMPORT_META_FILENAME {
        let result = self.process_import_meta(parser, ModulePathType::FileName);
        parser.add_presentational_dependency(Arc::new(ConstDependency::new(
          member_expr.span().into(),
          result.into(),
        )));
        return Some(true);
      } else {
        return None;
      }
    }

    None
  }

  fn optional_chaining(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: &OptChainExpr,
  ) -> Option<bool> {
    if !self.options.inject_import_meta_rstest_origin {
      return None;
    }

    let member = match &expr.base {
      OptChainBase::Call(call) => call.callee.as_member()?,
      OptChainBase::Member(member) => member.obj.as_member()?,
    };
    if !expr.optional
      || !member
        .obj
        .as_meta_prop()
        .is_some_and(|meta| meta.kind == MetaPropKind::ImportMeta)
      || member
        .prop
        .as_ident()
        .is_none_or(|property| property.sym != "rstest")
    {
      return None;
    }

    parser.add_presentational_dependency(Arc::new(ConstDependency::new(
      expr.span().into(),
      "undefined".into(),
    )));
    Some(true)
  }
}
