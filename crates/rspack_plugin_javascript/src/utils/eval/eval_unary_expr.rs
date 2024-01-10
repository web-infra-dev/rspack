use rspack_core::SpanExt;
use swc_core::ecma::ast::{UnaryExpr, UnaryOp};

use super::BasicEvaluatedExpression;
use crate::parser_plugin::JavascriptParserPlugin;
use crate::visitors::common_js_import_dependency_scanner::CommonJsImportDependencyScanner;

fn eval_typeof(
  scanner: &mut CommonJsImportDependencyScanner,
  expr: &UnaryExpr,
) -> Option<BasicEvaluatedExpression> {
  assert!(expr.op == UnaryOp::TypeOf);
  if let Some(ident) = expr.arg.as_ident()
    && let res = scanner.plugin_drive.evaluate_typeof(
      ident,
      expr.span.real_lo(),
      expr.span.hi().0,
      scanner.unresolved_ctxt,
    )
    && res.is_some()
  {
    return res;
  }

  // TODO: if let `MetaProperty`, `MemberExpression` ...
  None
}

pub fn eval_unary_expression(
  scanner: &mut CommonJsImportDependencyScanner,
  expr: &UnaryExpr,
) -> Option<BasicEvaluatedExpression> {
  match expr.op {
    UnaryOp::TypeOf => eval_typeof(scanner, expr),
    _ => None,
  }
}
