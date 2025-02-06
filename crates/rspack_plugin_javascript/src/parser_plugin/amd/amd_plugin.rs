use rspack_core::{ConstDependency, RuntimeGlobals, SpanExt};
use swc_core::ecma::ast::{CallExpr, Expr, MemberExpr};
use swc_core::{common::Spanned, ecma::ast::UnaryExpr};

use crate::utils::eval::{evaluate_to_identifier, evaluate_to_string, BasicEvaluatedExpression};
use crate::visitors::JavascriptParser;
use crate::JavascriptParserPlugin;

pub struct AMDParserPlugin;

const DEFINE: &str = "define";
const REQUIRE: &str = "require";
const DEFINE_AMD: &str = "define.amd";
const REQUIRE_AMD: &str = "require.amd";

impl JavascriptParserPlugin for AMDParserPlugin {
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
      return Some(true);
    }
    None
  }

  fn member(
    &self,
    parser: &mut JavascriptParser,
    expr: &MemberExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == "require.version" {
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
    if for_name == "requirejs.onError" {
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

    // AMD
    if for_name == "define.amd" || for_name == "require.amd" {
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

  fn identifier(
    &self,
    parser: &mut JavascriptParser,
    ident: &swc_core::ecma::ast::Ident,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == DEFINE {
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          ident.span().real_lo(),
          ident.span().real_hi(),
          RuntimeGlobals::AMD_DEFINE.name().into(),
          Some(RuntimeGlobals::AMD_DEFINE),
        )));
      return Some(true);
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

  fn can_rename(&self, _parser: &mut JavascriptParser, for_name: &str) -> Option<bool> {
    if for_name == DEFINE {
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
      return Some(false);
    }
    None
  }
}
