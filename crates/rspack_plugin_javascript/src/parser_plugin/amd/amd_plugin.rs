use rspack_core::{ConstDependency, RuntimeGlobals};
use rspack_util::SpanExt;
use swc_core::{
  common::Spanned,
  ecma::ast::{CallExpr, Expr, MemberExpr, UnaryExpr},
};

use crate::{
  JavascriptParserPlugin,
  utils::eval::{BasicEvaluatedExpression, evaluate_to_identifier, evaluate_to_string},
  visitors::JavascriptParser,
};

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
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        call_expr.span.into(),
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
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        expr.span.into(),
        "\"0.0.0\"".into(),
        None,
      )));
      return Some(true);
    }
    if for_name == "requirejs.onError" {
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        expr.span.into(),
        RuntimeGlobals::UNCAUGHT_ERROR_HANDLER.name().into(),
        Some(RuntimeGlobals::UNCAUGHT_ERROR_HANDLER),
      )));
      return Some(true);
    }

    // AMD
    if for_name == "define.amd" || for_name == "require.amd" {
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        expr.span.into(),
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
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        expr.span.into(),
        "\"function\"".into(),
        None,
      )));
      return Some(true);
    }

    if for_name == DEFINE_AMD || for_name == REQUIRE_AMD {
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        expr.span.into(),
        "\"object\"".into(),
        None,
      )));
      return Some(true);
    }

    None
  }

  fn evaluate_typeof<'a>(
    &self,
    _parser: &mut JavascriptParser,
    expr: &'a UnaryExpr,
    for_name: &str,
  ) -> Option<BasicEvaluatedExpression<'a>> {
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
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        ident.span().into(),
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
    for_name: &str,
    start: u32,
    end: u32,
  ) -> Option<BasicEvaluatedExpression<'static>> {
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

  fn can_rename(&self, _parser: &mut JavascriptParser, for_name: &str) -> Option<bool> {
    if for_name == DEFINE {
      return Some(true);
    }
    None
  }

  fn rename(&self, parser: &mut JavascriptParser, expr: &Expr, for_name: &str) -> Option<bool> {
    if for_name == DEFINE {
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        expr.span().into(),
        RuntimeGlobals::AMD_DEFINE.name().into(),
        Some(RuntimeGlobals::AMD_DEFINE),
      )));
      return Some(false);
    }
    None
  }
}
