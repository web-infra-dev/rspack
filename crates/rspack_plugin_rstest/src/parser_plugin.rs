use std::sync::Arc;

use camino::Utf8PathBuf;
use rspack_core::{
  AsyncDependenciesBlock, BoxDependency, ConstDependency, DependencyRange, ImportAttributes,
  ImportPhase,
};
use rspack_plugin_javascript::{
  JavascriptParserPlugin,
  dependency::{CommonJsRequireDependency, ImportDependency, RequireHeaderDependency},
  try_extract_magic_comment,
  utils::{
    self,
    eval::{self},
  },
  visitors::{
    HookMemberExpression, Identifier, JavascriptParser, Statement, VariableDeclaration,
    create_traceable_error, expr_name,
  },
};
use rspack_util::{SpanExt, atom::Atom, json_stringify_str, swc::get_swc_next_comments};
use swc_next_ecma_ast::{
  Argument, Ast, CallExpression, ChainExpression, Expr, ExprData, GetSpan, IdentifierName,
  IdentifierReference, ImportDeclaration, ImportExpression, PropertyKeyData, Span, UnaryExpression,
  VariableDeclarator,
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

fn collect_arguments(ast: &Ast<'_>, call: CallExpression) -> Vec<Argument> {
  call
    .arguments(ast)
    .iter()
    .map(|id| ast.get_node_in_sub_range(id))
    .collect()
}

fn string_literal_value(ast: &Ast<'_>, expr: Expr) -> Option<String> {
  let ExprData::StringLiteral(literal) = ast.expr_data(expr) else {
    return None;
  };
  Some(
    ast
      .get_wtf8(literal.value(ast))
      .to_string_lossy()
      .into_owned(),
  )
}

fn call_member_identifiers(
  ast: &Ast<'_>,
  call: CallExpression,
) -> Option<(IdentifierReference, IdentifierName)> {
  let ExprData::MemberExpression(member) = ast.expr_data(call.callee(ast)) else {
    return None;
  };
  let ExprData::IdentifierReference(object) = ast.expr_data(member.object(ast)) else {
    return None;
  };
  let PropertyKeyData::IdentifierName(property) = ast.property_key_data(member.property(ast))
  else {
    return None;
  };
  Some((object, property))
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
    call_expr: CallExpression,
  ) -> Option<bool> {
    let ast = parser.ast.ast;
    let (require_ident, _) = call_member_identifiers(ast, call_expr)?;

    if parser
      .get_variable_info(&Atom::from(ast.get_utf8(require_ident.name(ast))))
      .is_some()
    {
      return None;
    }

    let args = collect_arguments(ast, call_expr);
    if !(1..=2).contains(&args.len()) {
      return None;
    }

    let first_arg = *args.first()?;
    if first_arg.is_spread_element(ast)
      || self.has_ignore_comment(parser, call_expr.span(ast), first_arg.span(ast))
    {
      return None;
    }

    if args.get(1).is_some_and(|arg| arg.is_spread_element(ast)) {
      return None;
    }

    let resource_path = parser.resource_data.path()?;
    let origin_path = resource_path.as_str().to_string();

    let last_arg = args.last().expect("args has at least one element");
    parser.add_presentational_dependency(Arc::new(RstestRequireResolveOriginDependency::new(
      call_expr.callee(ast).span(ast).into(),
      last_arg.span(ast).real_hi(),
      origin_path,
    )));

    // Returning `Some(true)` short-circuits the default walker for this call,
    // so preserve dependency collection for nested expressions in arguments.
    parser.walk_arguments(args.into_iter());
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
    call_expr: CallExpression,
  ) -> Option<bool> {
    let ast = parser.ast.ast;
    let args = collect_arguments(ast, call_expr);
    match args.len() {
      1 => {
        let first_arg = args[0];
        if let Some(first_arg_expr) = first_arg.as_expr(ast)
          && let Some(value) = string_literal_value(ast, first_arg_expr)
        {
          let first_arg_range: DependencyRange = first_arg.span(ast).into();
          let loc = parser.to_dependency_location(first_arg_range);
          let dep = CommonJsRequireDependency::new(
            value,
            first_arg_range,
            Some(call_expr.span(ast).into()),
            parser.in_try,
            loc,
          );
          parser.add_dependency(BoxDependency::new(dep));

          let callee_range = call_expr.callee(ast).span(ast).into();
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
            call_expr.span(ast).into(),
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
    call_expr: CallExpression,
  ) -> Option<bool> {
    let ast = parser.ast.ast;
    let args = collect_arguments(ast, call_expr);
    match args.len() {
      1 => {
        let first_arg = args[0];
        if let Some(first_arg_expr) = first_arg.as_expr(ast)
          && let Some(value) = string_literal_value(ast, first_arg_expr)
        {
          let mut attrs = ImportAttributes::default();
          attrs.insert("rstest".to_string(), "importActual".to_string());

          let imported_span = first_arg.span(ast);
          let range = call_expr.span(ast).into();
          let dep = BoxDependency::new(ImportDependency::new(
            Atom::from(value.as_str()),
            range,
            Some(attrs),
            ImportPhase::Evaluation,
            parser.in_try,
            get_swc_next_comments(parser.ast.comments, imported_span.start, imported_span.end),
          ));

          let loc = parser.to_dependency_location(range);
          let block = AsyncDependenciesBlock::new(
            *parser.module_identifier,
            loc,
            None,
            vec![dep],
            Some(value),
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
            call_expr.span(ast).into(),
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
    mock_call_expr: CallExpression,
  ) -> Option<String> {
    let ast = parser.ast.ast;
    let first_arg = collect_arguments(ast, mock_call_expr).into_iter().next()?;
    let first_arg_expr = first_arg.as_expr(ast)?;

    if let ExprData::ImportExpression(import_call) = ast.expr_data(first_arg_expr) {
      parser.tag_variable::<bool>(
        self
          .compose_rstest_import_call_key(import_call.span(ast))
          .into(),
        RSTEST_MOCK_FIRST_ARG_TAG,
        Some(true),
      );
      string_literal_value(ast, import_call.source(ast))
    } else {
      string_literal_value(ast, first_arg_expr)
    }
  }

  #[allow(clippy::too_many_arguments)]
  fn process_mock(
    &self,
    parser: &mut JavascriptParser,
    call_expr: CallExpression,
    hoist: bool,
    is_esm: bool,
    method: MockMethod,
    has_b: bool,
    test_api_import_source_order: Option<i32>,
  ) {
    let ast = parser.ast.ast;
    let args = collect_arguments(ast, call_expr);
    match args.len() {
      1 => {
        let first_arg = args[0];
        let first_arg_lit_str = self.handle_mock_first_arg(parser, call_expr);

        if let Some(lit_str) = first_arg_lit_str {
          let dep = MockModuleIdDependency::new(
            lit_str.clone(),
            first_arg.span(ast).into(),
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
              call_expr.span(ast).into(),
              call_expr.callee(ast).span(ast).into(),
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
              Some(first_arg.span(ast).real_hi())
            })
            .with_test_api_import_source_order(test_api_import_source_order),
          ));

          if has_b {
            let first_arg_span = first_arg.span(ast);
            let second_arg = Span::new(first_arg_span.end, first_arg_span.end);
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
        let first_arg = args[0];
        let second_arg = args[1];

        if first_arg.is_spread_element(ast) || second_arg.is_spread_element(ast) {
          return;
        }

        let lit_str = self.handle_mock_first_arg(parser, call_expr);

        if let Some(lit_str) = lit_str {
          let module_dep = MockModuleIdDependency::new(
            lit_str.clone(),
            first_arg.span(ast).into(),
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
              call_expr.span(ast).into(),
              call_expr.callee(ast).span(ast).into(),
              lit_str,
              hoist,
              method,
            )
            // 2-arg `rs.mock('X', factory)`: append request after the factory.
            .with_request_arg_end(Some(second_arg.span(ast).real_hi()))
            .with_test_api_import_source_order(test_api_import_source_order),
          ));
          parser.add_dependency(BoxDependency::new(module_dep));
        } else {
          parser.add_error(
            create_traceable_error(
              "Invalid function call".into(),
              "`rs.mock` function expects a string literal as the first argument".into(),
              parser.source().to_string(),
              call_expr.span(ast).into(),
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
            call_expr.span(ast).into(),
          )
          .into(),
        );
      }
    }
  }

  fn hoisted(
    &self,
    parser: &mut JavascriptParser,
    call_expr: CallExpression,
    statement_span: Option<Span>,
    test_api_import_source_order: Option<i32>,
  ) -> Option<bool> {
    let ast = parser.ast.ast;
    match call_expr.arguments(ast).len() {
      1 => {
        let call_span = call_expr.span(ast);
        let callee_span = call_expr.callee(ast).span(ast);
        let dep = if let Some(stmt_span) = statement_span {
          MockMethodDependency::new_with_statement_range(
            call_span.into(),
            callee_span.into(),
            stmt_span.into(),
            call_span.real_lo().to_string(),
            true,
            MockMethod::Hoisted,
          )
        } else {
          MockMethodDependency::new(
            call_span.into(),
            callee_span.into(),
            call_span.real_lo().to_string(),
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
            call_expr.span(ast).into(),
          )
          .into(),
        );
        Some(false)
      }
    }
  }

  fn reset_modules(
    &self,
    parser: &mut JavascriptParser,
    call_expr: CallExpression,
  ) -> Option<bool> {
    let ast = parser.ast.ast;
    match call_expr.arguments(ast).len() {
      0 => {
        parser.add_presentational_dependency(Arc::new(ConstDependency::new(
          call_expr.callee(ast).span(ast).into(),
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
            call_expr.span(ast).into(),
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
    call_expr: CallExpression,
    is_esm: bool,
  ) -> Option<bool> {
    let ast = parser.ast.ast;
    let args = collect_arguments(ast, call_expr);
    match args.len() {
      1 => {
        let first_arg = args[0];
        if let Some(first_arg_expr) = first_arg.as_expr(ast) {
          if let Some(value) = string_literal_value(ast, first_arg_expr) {
            if let Some(mocked_target) = self.calc_mocked_target(&value).as_std_path().to_str() {
              if is_esm {
                let imported_span = first_arg.span(ast);

                let mut attrs = ImportAttributes::default();
                attrs.insert("rstest".to_string(), "importMock".to_string());
                let range = call_expr.span(ast).into();
                let dep = BoxDependency::new(ImportDependency::new(
                  Atom::from(mocked_target),
                  range,
                  Some(attrs),
                  ImportPhase::Evaluation,
                  parser.in_try,
                  get_swc_next_comments(
                    parser.ast.comments,
                    imported_span.start,
                    imported_span.end,
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
                let first_arg_range = first_arg.span(ast).into();
                let loc = parser.to_dependency_location(first_arg_range);
                let dep: CommonJsRequireDependency = CommonJsRequireDependency::new(
                  mocked_target.to_string(),
                  first_arg_range,
                  Some(call_expr.span(ast).into()),
                  parser.in_try,
                  loc,
                );

                let callee_range = call_expr.callee(ast).span(ast).into();
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
            call_expr.span(ast).into(),
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

  fn compose_rstest_import_call_key(&self, span: Span) -> String {
    format!(
      "rstest_strip_import_call {} {}",
      span.real_lo(),
      span.real_hi(),
    )
  }

  fn handle_rstest_method_call(
    &self,
    parser: &mut JavascriptParser,
    call_expr: CallExpression,
    ident: IdentifierReference,
    prop: IdentifierName,
    statement_span: Option<Span>,
  ) -> Option<bool> {
    let ast = parser.ast.ast;
    let ident_name_str = ast.get_utf8(ident.name(ast));
    let property_name = ast.get_utf8(prop.name(ast));
    let ident_name = Atom::from(ident_name_str);
    let test_api_import_source_order = parser
      .get_tag_data::<i32>(&ident_name, RSTEST_API_IMPORT_TAG)
      .copied();

    // Check if this is a global variable (free variable) or an ESM import
    let is_global = !parser.is_variable_defined(&ident_name);

    // Skip global variables if globals option is disabled
    if is_global && !self.options.globals {
      return None;
    }

    match (ident_name_str, property_name) {
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
    _statement: ImportDeclaration,
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
    _expr: VariableDeclarator,
    stmt: VariableDeclaration,
  ) -> Option<bool> {
    let ast = parser.ast.ast;
    for decl in stmt.declarators(ast) {
      if let Some(init) = decl.init(ast) {
        let call_expr = match ast.expr_data(init) {
          ExprData::CallExpression(call) => Some(call),
          ExprData::AwaitExpression(await_expr) => match ast.expr_data(await_expr.argument(ast)) {
            ExprData::CallExpression(call) => Some(call),
            _ => None,
          },
          _ => None,
        };

        if let Some(call_expr) = call_expr
          && let Some((obj_ident, prop_ident)) = call_member_identifiers(ast, call_expr)
        {
          return self.handle_rstest_method_call(
            parser,
            call_expr,
            obj_ident,
            prop_ident,
            Some(stmt.span(ast)),
          );
        }
      }
    }

    None
  }

  fn statement(&self, parser: &mut JavascriptParser<'p>, stmt: Statement) -> Option<bool> {
    if !self.options.hoist_mock_module {
      return None;
    }

    let ast = parser.ast.ast;
    let Statement::Expr(expr_stmt) = stmt else {
      return None;
    };
    let ExprData::CallExpression(call_expr) = ast.expr_data(expr_stmt.expression(ast)) else {
      return None;
    };

    if let Some((obj_ident, prop_ident)) = call_member_identifiers(ast, call_expr) {
      return self.handle_rstest_method_call(parser, call_expr, obj_ident, prop_ident, None);
    }

    None
  }

  fn call(
    &self,
    parser: &mut JavascriptParser<'p>,
    call_expr: CallExpression,
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
    import_expr: ImportExpression,
    import_then: Option<CallExpression>,
    _members: Option<(&[Atom], bool)>,
  ) -> Option<bool> {
    let ast = parser.ast.ast;
    let import_span = import_expr.span(ast);
    let tag_data = parser.get_tag_data::<bool>(
      &self.compose_rstest_import_call_key(import_span).into(),
      RSTEST_MOCK_FIRST_ARG_TAG,
    );
    if tag_data.is_some() {
      return Some(true);
    }

    if self.options.inject_dynamic_import_origin {
      // Only handle the regular evaluation phase. `import.defer(...)` and
      // `import.source(...)` carry phase semantics that rstest's runtime
      // does not implement, and the default `ImportParserPlugin` enforces
      // the `experiments.deferImport` gate which we must not bypass.
      if import_expr.phase(ast).is_some() {
        return None;
      }

      // Mirror `ImportParserPlugin.import_call`'s `/* webpackIgnore: true */`
      // bailout so authors can opt out of rewriting on a per-call basis.
      let source = import_expr.source(ast);
      let source_span = source.span(ast);
      let magic = try_extract_magic_comment(parser, import_span, source_span);
      if magic.get_ignore().unwrap_or_default() {
        return None;
      }

      let param = parser.evaluate_expression(source);
      if param.is_string() {
        return None;
      }

      let resource_path = parser.resource_data.path()?;
      let origin_path = resource_path.as_str().to_string();

      let options = import_expr.options(ast);
      let args_end = options.unwrap_or(source).span(ast).real_hi();
      let has_attributes = options.is_some();
      let import_keyword_span = Span::new(import_span.start, import_span.start + 6);

      parser.add_presentational_dependency(Arc::new(RstestDynamicImportOriginDependency::new(
        import_keyword_span.into(),
        args_end,
        has_attributes,
        origin_path,
      )));

      // Returning `Some(true)` short-circuits the parser's walk of this
      // `import()` node, so we must
      // walk nested expressions ourselves — otherwise `require(...)` or
      // `import()` calls inside the specifier or the `.then` callback get
      // dropped from the dependency graph.
      parser.walk_expression(source);
      if let Some(options) = options {
        parser.walk_expression(options);
      }
      if let Some(import_then) = import_then {
        parser.walk_arguments(collect_arguments(ast, import_then).into_iter());
      }

      return Some(true);
    }

    None
  }

  fn call_member_chain(
    &self,
    parser: &mut JavascriptParser<'p>,
    call_expr: CallExpression,
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
      && let Some((ident, _)) = call_member_identifiers(parser.ast.ast, call_expr)
    {
      let var_name = parser.ast.ast.get_utf8(ident.name(parser.ast.ast));
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
    _ident: &Identifier,
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
    expr: UnaryExpression,
    for_name: &str,
  ) -> Option<utils::eval::BasicEvaluatedExpression<'p>> {
    let span = expr.span(parser.ast.ast);
    if for_name == IMPORT_META_RSTEST && self.import_meta_rstest_expression(parser).is_some() {
      return Some(eval::BasicEvaluatedExpression::with_range(
        span.real_lo(),
        span.real_hi(),
      ));
    }

    if self.options.import_meta_path_name {
      let mut evaluated = None;
      if for_name == IMPORT_META_DIRNAME || for_name == IMPORT_META_FILENAME {
        evaluated = Some("string".to_string());
      }
      return evaluated.map(|e| eval::evaluate_to_string(e, span.real_lo(), span.real_hi()));
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
    unary_expr: UnaryExpression,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == IMPORT_META_RSTEST
      && let Some(expression) = self.import_meta_rstest_expression(parser)
    {
      parser.add_presentational_dependency(Arc::new(ConstDependency::new(
        unary_expr.span(parser.ast.ast).into(),
        format!("typeof ({expression})").into(),
      )));
      return Some(true);
    }

    if self.options.import_meta_path_name {
      if for_name == IMPORT_META_DIRNAME || for_name == IMPORT_META_FILENAME {
        parser.add_presentational_dependency(Arc::new(ConstDependency::new(
          unary_expr.span(parser.ast.ast).into(),
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
    member_expr: HookMemberExpression,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == IMPORT_META_RSTEST
      && let Some(expression) = self.import_meta_rstest_expression(parser)
    {
      // TODO: Replace this Rstest-specific parser rewrite with
      // DefinePlugin.runtimeValue once Rspack supports the webpack API.
      parser.add_presentational_dependency(Arc::new(ConstDependency::new(
        member_expr.span(parser.ast.ast).into(),
        expression.into(),
      )));
      return Some(true);
    }

    if self.options.import_meta_path_name {
      if for_name == IMPORT_META_DIRNAME {
        let result = self.process_import_meta(parser, ModulePathType::DirName);
        parser.add_presentational_dependency(Arc::new(ConstDependency::new(
          member_expr.span(parser.ast.ast).into(),
          result.into(),
        )));
        return Some(true);
      } else if for_name == IMPORT_META_FILENAME {
        let result = self.process_import_meta(parser, ModulePathType::FileName);
        parser.add_presentational_dependency(Arc::new(ConstDependency::new(
          member_expr.span(parser.ast.ast).into(),
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
    expr: ChainExpression,
  ) -> Option<bool> {
    if !self.options.inject_import_meta_rstest_origin {
      return None;
    }

    let ast = parser.ast.ast;
    let member = match ast.expr_data(expr.expression(ast)) {
      ExprData::CallExpression(call) if call.optional(ast) => {
        let ExprData::MemberExpression(member) = ast.expr_data(call.callee(ast)) else {
          return None;
        };
        member
      }
      ExprData::MemberExpression(optional_member) if optional_member.optional(ast) => {
        let ExprData::MemberExpression(member) = ast.expr_data(optional_member.object(ast)) else {
          return None;
        };
        member
      }
      _ => return None,
    };

    let ExprData::MetaProperty(meta) = ast.expr_data(member.object(ast)) else {
      return None;
    };
    let PropertyKeyData::IdentifierName(property) = ast.property_key_data(member.property(ast))
    else {
      return None;
    };
    if ast.get_utf8(meta.meta(ast).name(ast)) != "import"
      || ast.get_utf8(meta.property(ast).name(ast)) != "meta"
      || ast.get_utf8(property.name(ast)) != "rstest"
    {
      return None;
    }

    parser.add_presentational_dependency(Arc::new(ConstDependency::new(
      expr.span(ast).into(),
      "undefined".into(),
    )));
    Some(true)
  }
}
