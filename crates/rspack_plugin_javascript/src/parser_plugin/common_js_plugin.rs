use rspack_core::{ConstDependency, RuntimeGlobals, RuntimeRequirementsDependency};
use swc_core::ecma::ast::{Expr, MemberExpr};

use super::JavascriptParserPlugin;
use crate::{
  utils::eval::{BasicEvaluatedExpression, evaluate_to_identifier},
  visitors::{JavascriptParser, expr_matcher, expr_name},
};

pub struct CommonJsPlugin;

impl JavascriptParserPlugin for CommonJsPlugin {
  fn evaluate_identifier(
    &self,
    _parser: &mut JavascriptParser,
    for_name: &str,
    start: u32,
    end: u32,
  ) -> Option<BasicEvaluatedExpression<'static>> {
    if for_name == expr_name::MODULE_HOT {
      Some(evaluate_to_identifier(
        expr_name::MODULE_HOT.to_string(),
        expr_name::MODULE.to_string(),
        None,
        start,
        end,
      ))
    } else {
      None
    }
  }

  fn r#typeof(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::UnaryExpr,
    _for_name: &str,
  ) -> Option<bool> {
    if expr_matcher::is_module(&*expr.arg) && parser.is_unresolved_ident("module") {
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          expr.span.into(),
          "'object'".into(),
          None,
        )));
      Some(true)
    } else {
      None
    }
  }

  fn member(&self, parser: &mut JavascriptParser, expr: &MemberExpr, _name: &str) -> Option<bool> {
    // FIXME: delete this `.clone` after extract expression
    let expr = Expr::Member(expr.clone());
    if expr_matcher::is_module_id(&expr) {
      parser
        .presentational_dependencies
        .push(Box::new(RuntimeRequirementsDependency::new(
          RuntimeGlobals::MODULE_ID,
        )));

      parser.build_info.module_concatenation_bailout = Some(RuntimeGlobals::MODULE_ID.to_string());
      Some(true)
    } else if expr_matcher::is_module_loaded(&expr) {
      parser
        .presentational_dependencies
        .push(Box::new(RuntimeRequirementsDependency::new(
          RuntimeGlobals::MODULE_LOADED,
        )));
      parser.build_info.module_concatenation_bailout =
        Some(RuntimeGlobals::MODULE_LOADED.to_string());
      Some(true)
    } else {
      None
    }
  }
}
