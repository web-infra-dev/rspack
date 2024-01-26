use rspack_core::{ConstDependency, RuntimeGlobals, RuntimeRequirementsDependency, SpanExt};
use swc_core::ecma::ast::{Expr, MemberExpr};

use super::JavascriptParserPlugin;
use crate::visitors::{expr_matcher, JavascriptParser};

pub struct CommonJsPlugin;

const MODULE_NAME: &str = "module";

impl JavascriptParserPlugin for CommonJsPlugin {
  fn r#typeof(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::UnaryExpr,
  ) -> Option<bool> {
    if expr_matcher::is_module(&expr.arg) && parser.is_unresolved_ident("module") {
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

  fn member(&self, parser: &mut JavascriptParser, expr: &MemberExpr) -> Option<bool> {
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

  // FIXME: `identifier(` should be delete and align `commonjsSelfdependency` with webpack
  fn identifier(
    &self,
    parser: &mut JavascriptParser,
    ident: &swc_core::ecma::ast::Ident,
  ) -> Option<bool> {
    if ident.sym != MODULE_NAME {
      None
    } else if parser.has_module_ident {
      Some(true)
    } else if parser.is_unresolved_ident(MODULE_NAME) {
      parser
        .presentational_dependencies
        .push(Box::new(RuntimeRequirementsDependency::new(
          RuntimeGlobals::MODULE,
        )));
      parser.has_module_ident = true;
      Some(true)
    } else {
      None
    }
  }
}
