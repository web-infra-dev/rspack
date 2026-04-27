use camino::Utf8PathBuf;
use rspack_core::{
  AsyncDependenciesBlock, ConstDependency, DependencyRange, ImportAttributes, ImportPhase,
};
use rspack_plugin_javascript::{
  JavascriptParserPlugin,
  dependency::{CommonJsRequireDependency, ImportDependency, RequireHeaderDependency},
  magic_comment::try_extract_magic_comment,
  utils::{
    self,
    eval::{self},
  },
  visitors::{JavascriptParser, Statement, VariableDeclaration, create_traceable_error},
};
use rspack_util::{SpanExt, atom::Atom, json_stringify_str, swc::get_swc_comments};
use swc_core::{
  common::{Span, Spanned},
  ecma::ast::{CallExpr, Callee, Ident, MemberExpr, UnaryExpr},
};

static RSTEST_MOCK_FIRST_ARG_TAG: &str = "strip the import call from the first arg of mock series";

use crate::{
  dynamic_import_origin_dependency::RstestDynamicImportOriginDependency,
  mock_method_dependency::{MockMethod, MockMethodDependency},
  mock_module_id_dependency::MockModuleIdDependency,
  module_path_name_dependency::{ModulePathNameDependency, NameType},
};

const DIR_NAME: &str = "__dirname";
const FILE_NAME: &str = "__filename";
const IMPORT_META_DIRNAME: &str = "import.meta.dirname";
const IMPORT_META_FILENAME: &str = "import.meta.filename";
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
  /// When `module.parser.javascript.importDynamic` is `false` and the dynamic
  /// import specifier is not a plain string literal, rewrite the callee to the
  /// configured `output.importFunctionName` and append the source module's
  /// absolute path as an extra argument so the runtime can resolve relative
  /// specifiers against it.
  pub inject_dynamic_import_origin: bool,
  /// Snapshot of `module.parser.javascript*.importDynamic` for this module
  /// type. Used by the dynamic-import-origin rewrite to gate behavior on
  /// `Some(false)`.
  pub import_dynamic: Option<bool>,
  /// Snapshot of `output.importFunctionName`. Used by the
  /// dynamic-import-origin rewrite to substitute the `import` callee.
  pub import_function_name: String,
}

impl Default for RstestParserPluginOptions {
  fn default() -> Self {
    Self {
      module_path_name: false,
      hoist_mock_module: false,
      import_meta_path_name: false,
      manual_mock_root: String::new(),
      globals: true,
      inject_dynamic_import_origin: false,
      import_dynamic: None,
      import_function_name: String::new(),
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
            None,
          );
          parser.add_dependency(Box::new(dep));

          let callee_range = call_expr.callee.span().into();
          let loc = parser.to_dependency_location(callee_range);
          parser.add_presentational_dependency(Box::new(RequireHeaderDependency::new(
            callee_range,
            loc,
          )));

          parser.add_presentational_dependency(Box::new(ConstDependency::new(
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
          let dep = Box::new(ImportDependency::new(
            lit.value.to_atom_lossy().into_owned(),
            range,
            None,
            Some(attrs),
            ImportPhase::Evaluation,
            parser.in_try,
            get_swc_comments(
              parser.comments,
              imported_span.span().lo,
              imported_span.span().hi,
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
          parser.add_dependency(Box::new(dep));

          parser.add_presentational_dependency(Box::new(MockMethodDependency::new(
            call_expr.span().into(),
            call_expr.callee.span().into(),
            lit_str.clone(),
            hoist,
            method,
          )));

          if has_b {
            let second_arg = Span::new(
              first_arg.span().hi() + swc_core::common::BytePos(0),
              first_arg.span().hi() + swc_core::common::BytePos(0),
            );
            parser.add_dependency(Box::new(MockModuleIdDependency::new(
              format!("{MOCK_TARGET_REQUEST_PREFIX}{lit_str}"),
              second_arg.into(),
              false,
              true,
              if is_esm {
                rspack_core::DependencyCategory::Esm
              } else {
                rspack_core::DependencyCategory::CommonJS
              },
              None,
            )));
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

          parser.add_presentational_dependency(Box::new(MockMethodDependency::new(
            call_expr.span().into(),
            call_expr.callee.span().into(),
            lit_str,
            hoist,
            method,
          )));
          parser.add_dependency(Box::new(module_dep));
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
        parser.add_presentational_dependency(Box::new(dep));
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
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
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
                let dep = Box::new(ImportDependency::new(
                  Atom::from(mocked_target),
                  range,
                  None,
                  Some(attrs),
                  ImportPhase::Evaluation,
                  parser.in_try,
                  get_swc_comments(
                    parser.comments,
                    imported_span.span().lo,
                    imported_span.span().hi,
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
                  None,
                );

                let callee_range = call_expr.callee.span().into();
                let loc = parser.to_dependency_location(callee_range);
                parser.add_presentational_dependency(Box::new(RequireHeaderDependency::new(
                  callee_range,
                  loc,
                )));

                parser.add_dependency(Box::new(dep));
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
    prop: &swc_core::ecma::ast::IdentName,
    statement_span: Option<Span>,
  ) -> Option<bool> {
    // Check if this is a global variable (free variable) or an ESM import
    let is_global = !parser.is_variable_defined(&ident.sym);

    // Skip global variables if globals option is disabled
    if is_global && !self.options.globals {
      return None;
    }

    match (ident.sym.as_str(), prop.sym.as_str()) {
      // rs.mock
      ("rs" | "rstest", "mock") => {
        self.process_mock(parser, call_expr, true, true, MockMethod::Mock, true);
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
        );
        Some(false)
      }
      // rs.doMock
      ("rs" | "rstest", "doMock") => {
        self.process_mock(parser, call_expr, false, true, MockMethod::DoMock, true);
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
        self.process_mock(parser, call_expr, true, true, MockMethod::Unmock, false);
        Some(true)
      }
      // rs.doUnmock
      ("rs" | "rstest", "doUnmock") => {
        self.process_mock(parser, call_expr, false, true, MockMethod::Unmock, false);
        Some(true)
      }
      // rs.unmockRequire
      ("rs" | "rstest", "unmockRequire") => {
        self.process_mock(parser, call_expr, true, false, MockMethod::Unmock, false);
        Some(true)
      }
      // rs.doUnmockRequire
      ("rs" | "rstest", "doUnmockRequire") => {
        self.process_mock(parser, call_expr, false, false, MockMethod::Unmock, false);
        Some(true)
      }
      // rs.resetModules
      ("rs" | "rstest", "resetModules") => self.reset_modules(parser, call_expr),
      // rs.hoisted
      ("rs" | "rstest", "hoisted") => self.hoisted(parser, call_expr, statement_span),
      _ => {
        // Not a mock module, continue.
        None
      }
    }
  }
}

#[rspack_plugin_javascript::implemented_javascript_parser_hooks]
impl JavascriptParserPlugin for RstestParserPlugin {
  fn declarator(
    &self,
    parser: &mut JavascriptParser,
    _expr: &swc_core::ecma::ast::VarDeclarator,
    stmt: VariableDeclaration<'_>,
  ) -> Option<bool> {
    for decl in stmt.declarators() {
      if let Some(init) = &decl.init {
        let call_expr = match init.as_call() {
          Some(call) => Some(call),
          None => init
            .as_await_expr()
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

  fn statement(&self, parser: &mut JavascriptParser, stmt: Statement) -> Option<bool> {
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

  fn import_call(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
    import_then: Option<&CallExpr>,
    _members: Option<(&[Atom], bool)>,
  ) -> Option<bool> {
    let first_arg = self.handle_mock_first_arg(parser, call_expr);
    if first_arg.is_some() {
      let tag_data = parser.get_tag_data(
        &self.compose_rstest_import_call_key(call_expr).into(),
        RSTEST_MOCK_FIRST_ARG_TAG,
      );

      if tag_data.is_some() {
        return Some(true);
      }
    }

    if self.options.inject_dynamic_import_origin
      && matches!(self.options.import_dynamic, Some(false))
    {
      // Only handle the regular evaluation phase. `import.defer(...)` and
      // `import.source(...)` carry phase semantics that rstest's runtime
      // does not implement, and the default `ImportParserPlugin` enforces
      // the `experiments.deferImport` gate which we must not bypass.
      let import_node = call_expr.callee.as_import()?;
      if !matches!(
        import_node.phase,
        swc_core::ecma::ast::ImportPhase::Evaluation
      ) {
        return None;
      }

      // Native `import()` only accepts 1-2 args; appending a third argument
      // would produce a SyntaxError at parse time. Skip injection when no
      // custom `importFunctionName` is configured and let the default
      // `ImportParserPlugin` handle the call as before.
      if self.options.import_function_name == "import" {
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

      let param = parser.evaluate_expression(arg.expr.as_ref());
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

      parser.add_presentational_dependency(Box::new(RstestDynamicImportOriginDependency::new(
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
    parser: &mut JavascriptParser,
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
    parser: &mut rspack_plugin_javascript::visitors::JavascriptParser,
    _ident: &Ident,
    for_name: &str,
  ) -> Option<bool> {
    if self.options.module_path_name {
      match for_name {
        DIR_NAME => {
          parser.add_presentational_dependency(Box::new(ModulePathNameDependency::new(
            NameType::DirName,
          )));
          return Some(true);
        }
        FILE_NAME => {
          parser.add_presentational_dependency(Box::new(ModulePathNameDependency::new(
            NameType::FileName,
          )));
          return Some(true);
        }
        _ => return None,
      }
    }

    None
  }

  fn evaluate_typeof<'a>(
    &self,
    _parser: &mut JavascriptParser,
    expr: &'a UnaryExpr,
    for_name: &str,
  ) -> Option<utils::eval::BasicEvaluatedExpression<'a>> {
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
    parser: &mut JavascriptParser,
    for_name: &str,
    start: u32,
    end: u32,
  ) -> Option<eval::BasicEvaluatedExpression<'static>> {
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
    parser: &mut JavascriptParser,
    unary_expr: &UnaryExpr,
    for_name: &str,
  ) -> Option<bool> {
    if self.options.import_meta_path_name {
      if for_name == IMPORT_META_DIRNAME || for_name == IMPORT_META_FILENAME {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
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
    parser: &mut JavascriptParser,
    member_expr: &MemberExpr,
    for_name: &str,
  ) -> Option<bool> {
    if self.options.import_meta_path_name {
      if for_name == IMPORT_META_DIRNAME {
        let result = self.process_import_meta(parser, ModulePathType::DirName);
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          member_expr.span().into(),
          result.into(),
        )));
        return Some(true);
      } else if for_name == IMPORT_META_FILENAME {
        let result = self.process_import_meta(parser, ModulePathType::FileName);
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
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
}
