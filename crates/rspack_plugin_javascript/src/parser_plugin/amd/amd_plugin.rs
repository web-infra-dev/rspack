use std::sync::Arc;

use rspack_core::{ConstDependency, RuntimeGlobals, RuntimeRequirementsDependency};
use rspack_util::SpanExt;
use swc_next_ecma_ast::{CallExpression, Expr, GetSpan, UnaryExpression};

use crate::{
  JavascriptParserPlugin,
  utils::eval::{BasicEvaluatedExpression, evaluate_to_identifier, evaluate_to_string},
  visitors::{HookMemberExpression, Identifier, JavascriptParser},
};

pub struct AMDParserPlugin;

const DEFINE: &str = "define";
const REQUIRE: &str = "require";
const DEFINE_AMD: &str = "define.amd";
const REQUIRE_AMD: &str = "require.amd";

#[rspack_macros::implemented_javascript_parser_hooks]
impl<'p, 'a> JavascriptParserPlugin<'p, 'a> for AMDParserPlugin {
  fn call(
    &self,
    parser: &mut JavascriptParser<'p>,
    call_expr: CallExpression,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == "require.config" || for_name == "requirejs.config" {
      let ast = parser.ast.ast;
      parser.add_presentational_dependency(Arc::new(ConstDependency::new(
        call_expr.span(ast).into(),
        "undefined".into(),
      )));
      return Some(true);
    }
    None
  }

  fn member(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: HookMemberExpression,
    for_name: &str,
  ) -> Option<bool> {
    let ast = parser.ast.ast;
    if for_name == "require.version" {
      parser.add_presentational_dependency(Arc::new(ConstDependency::new(
        expr.span(ast).into(),
        "\"0.0.0\"".into(),
      )));
      return Some(true);
    }
    if for_name == "requirejs.onError" {
      parser.add_presentational_dependency(Arc::new(RuntimeRequirementsDependency::new(
        expr.span(ast).into(),
        RuntimeGlobals::UNCAUGHT_ERROR_HANDLER,
      )));
      return Some(true);
    }

    // AMD
    if for_name == "define.amd" || for_name == "require.amd" {
      parser.add_presentational_dependency(Arc::new(RuntimeRequirementsDependency::new(
        expr.span(ast).into(),
        RuntimeGlobals::AMD_OPTIONS,
      )));
      return Some(true);
    }
    None
  }

  // The following is the logic from AMDPlugin, which mainly applies
  // AMDDefineDependencyParserPlugin and AMDRequireDependenciesBlockParserPlugin.
  // It also has some require.js related logic. I moved the logic here
  // to avoid creating a `AMDPlugin` with just a few lines of code.

  fn r#typeof(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: UnaryExpression,
    for_name: &str,
  ) -> Option<bool> {
    let ast = parser.ast.ast;
    if for_name == DEFINE || for_name == REQUIRE {
      parser.add_presentational_dependency(Arc::new(ConstDependency::new(
        expr.span(ast).into(),
        "\"function\"".into(),
      )));
      return Some(true);
    }

    if for_name == DEFINE_AMD || for_name == REQUIRE_AMD {
      parser.add_presentational_dependency(Arc::new(ConstDependency::new(
        expr.span(ast).into(),
        "\"object\"".into(),
      )));
      return Some(true);
    }

    None
  }

  fn evaluate_typeof(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: UnaryExpression,
    for_name: &str,
  ) -> Option<BasicEvaluatedExpression<'p>> {
    let span = expr.span(parser.ast.ast);
    if for_name == DEFINE || for_name == REQUIRE {
      return Some(evaluate_to_string(
        "function".to_string(),
        span.real_lo(),
        span.real_hi(),
      ));
    }

    if for_name == DEFINE_AMD || for_name == REQUIRE_AMD {
      return Some(evaluate_to_string(
        "object".to_string(),
        span.real_lo(),
        span.real_hi(),
      ));
    }

    None
  }

  fn identifier(
    &self,
    parser: &mut JavascriptParser<'p>,
    ident: &Identifier,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == DEFINE {
      parser.add_presentational_dependency(Arc::new(RuntimeRequirementsDependency::new(
        ident.span().into(),
        RuntimeGlobals::AMD_DEFINE,
      )));
      return Some(true);
    }
    None
  }

  fn evaluate_identifier(
    &self,
    _parser: &mut JavascriptParser<'p>,
    for_name: &str,
    _member_expr_info: Option<&crate::visitors::ExpressionExpressionInfo>,
    start: u32,
    end: u32,
  ) -> Option<BasicEvaluatedExpression<'p>> {
    if for_name == DEFINE_AMD {
      return Some(evaluate_to_identifier(
        for_name.into(),
        "define".into(),
        Some(true),
        start,
        end,
      ));
    }

    if for_name == REQUIRE_AMD {
      return Some(evaluate_to_identifier(
        for_name.into(),
        "require".into(),
        Some(true),
        start,
        end,
      ));
    }

    None
  }

  fn can_rename(&self, _parser: &mut JavascriptParser<'p>, for_name: &str) -> Option<bool> {
    if for_name == DEFINE {
      return Some(true);
    }
    None
  }

  fn rename(&self, parser: &mut JavascriptParser<'p>, expr: Expr, for_name: &str) -> Option<bool> {
    if for_name == DEFINE {
      let span = expr.span(parser.ast.ast);
      parser.add_presentational_dependency(Arc::new(RuntimeRequirementsDependency::new(
        span.into(),
        RuntimeGlobals::AMD_DEFINE,
      )));
      return Some(false);
    }
    None
  }
}
