use rspack_core::{ConstDependency, RuntimeGlobals, RuntimeRequirementsDependency, SpanExt};
use swc_core::ecma::ast::{Expr, MemberExpr};

use super::JavascriptParserPlugin;
use crate::visitors::{expr_matcher, expr_name, JavascriptParser};

pub struct CommonJsPlugin;

impl JavascriptParserPlugin for CommonJsPlugin {
  fn r#typeof(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::UnaryExpr,
    for_name: &str,
  ) -> Option<bool> {
    if expr_matcher::is_module(&expr.arg) && for_name == expr_name::MODULE {
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          expr.span.real_lo(),
          expr.span.real_hi(),
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
      Some(true)
    } else if expr_matcher::is_module_loaded(&expr) {
      parser
        .presentational_dependencies
        .push(Box::new(RuntimeRequirementsDependency::new(
          RuntimeGlobals::MODULE_LOADED,
        )));
      Some(true)
    } else {
      None
    }
  }
}
