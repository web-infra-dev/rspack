use rspack_core::{ConstDependency, RuntimeGlobals, SpanExt};
use swc_core::ecma::ast::{CallExpr, Expr, MemberExpr};
use swc_core::{common::Spanned, ecma::ast::UnaryExpr};

use super::JavascriptParserPlugin;
use crate::utils::eval::{evaluate_to_identifier, evaluate_to_string, BasicEvaluatedExpression};
use crate::visitors::{expr_matcher, JavascriptParser};

pub struct RequireJsStuffPlugin;

const DEFINE: &str = "define";
const REQUIRE: &str = "require";
const DEFINE_AMD: &str = "define.amd";
const REQUIRE_AMD: &str = "require.amd";

impl JavascriptParserPlugin for RequireJsStuffPlugin {
  fn call(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == "require.config" || for_name == "requirejs.config" {
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          call_expr.span.real_lo(),
          call_expr.span.real_hi(),
          "undefined".into(),
          None,
        )));
      Some(true)
    } else {
      None
    }
  }

  fn member(
    &self,
    parser: &mut JavascriptParser,
    expr: &MemberExpr,
    _for_name: &str,
  ) -> Option<bool> {
    if expr_matcher::is_require_version(expr) {
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          expr.span.real_lo(),
          expr.span.real_hi(),
          "\"0.0.0\"".into(),
          None,
        )));
      return Some(true);
    }

    if expr_matcher::is_require_onerror(expr) || expr_matcher::is_requirejs_onerror(expr) {
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          expr.span.real_lo(),
          expr.span.real_hi(),
          RuntimeGlobals::UNCAUGHT_ERROR_HANDLER.name().into(),
          Some(RuntimeGlobals::UNCAUGHT_ERROR_HANDLER),
        )));
      return Some(true);
    }

    // AMDPlugin
    if expr_matcher::is_define_amd(expr) || expr_matcher::is_require_amd(expr) {
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          expr.span.real_lo(),
          expr.span.real_hi(),
          RuntimeGlobals::AMD_OPTIONS.name().into(),
          Some(RuntimeGlobals::AMD_OPTIONS),
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
    parser: &mut JavascriptParser,
    expr: &UnaryExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == DEFINE || for_name == REQUIRE {
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          expr.span.real_lo(),
          expr.span.real_hi(),
          "\"function\"".into(),
          None,
        )));
      return Some(true);
    }

    if for_name == DEFINE_AMD || for_name == REQUIRE_AMD {
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          expr.span.real_lo(),
          expr.span.real_hi(),
          "\"object\"".into(),
          None,
        )));
      return Some(true);
    }

    None
  }

  fn evaluate_typeof(
    &self,
    _parser: &mut JavascriptParser,
    expr: &UnaryExpr,
    for_name: &str,
  ) -> Option<BasicEvaluatedExpression> {
    if for_name == DEFINE || for_name == REQUIRE {
      return Some(evaluate_to_string(
        "function".to_string(),
        expr.span.real_lo(),
        expr.span.real_hi(),
      ));
    }

    if for_name == DEFINE_AMD || for_name == REQUIRE_AMD {
      return Some(evaluate_to_string(
        "object".to_string(),
        expr.span.real_lo(),
        expr.span.real_hi(),
      ));
    }

    None
  }

  fn evaluate_identifier(
    &self,
    _parser: &mut JavascriptParser,
    ident: &str,
    start: u32,
    end: u32,
  ) -> Option<BasicEvaluatedExpression> {
    if ident == DEFINE_AMD {
      return Some(evaluate_to_identifier(
        ident.to_string(),
        "define".to_string(),
        Some(true),
        start,
        end,
      ));
    }

    if ident == REQUIRE_AMD {
      return Some(evaluate_to_identifier(
        ident.to_string(),
        "require".to_string(),
        Some(true),
        start,
        end,
      ));
    }

    None
  }

  fn can_rename(
    &self,
    _parser: &mut JavascriptParser,
    for_name: &str,
    is_parameter: bool,
  ) -> Option<bool> {
    if for_name == DEFINE && !is_parameter {
      return Some(true);
    }
    None
  }

  fn rename(&self, parser: &mut JavascriptParser, expr: &Expr, for_name: &str) -> Option<bool> {
    if for_name == DEFINE {
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          expr.span().real_lo(),
          expr.span().real_hi(),
          RuntimeGlobals::AMD_DEFINE.name().into(),
          Some(RuntimeGlobals::AMD_DEFINE),
        )));
      return Some(true);
    }
    None
  }
}
