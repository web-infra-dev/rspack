use rspack_core::{ConstDependency, RuntimeGlobals, RuntimeRequirementsDependency, SpanExt};
use swc_core::ecma::ast::{Expr, MemberExpr};

use super::JavascriptParserPlugin;
use crate::visitors::{expr_matcher, JavascriptParser};

pub struct CommonJsPlugin;

impl JavascriptParserPlugin for CommonJsPlugin {
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
