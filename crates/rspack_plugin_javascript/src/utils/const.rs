use rspack_core::ConstDependency;
use rspack_core::SpanExt;
use swc_core::common::Spanned;
use swc_core::ecma::ast::{BinExpr, BinaryOp};
use swc_core::ecma::visit::VisitWith;

use super::evaluate_expression;
use crate::visitors::common_js_import_dependency_scanner::CommonJsImportDependencyScanner;

pub fn expression_logic_operator(
  scanner: &mut CommonJsImportDependencyScanner,
  expr: &BinExpr,
) -> Option<bool> {
  if expr.op == BinaryOp::LogicalAnd || expr.op == BinaryOp::LogicalOr {
    let param = evaluate_expression(&expr.left);
    let boolean = param.as_bool();
    let Some(bool) = boolean else {
      return None;
    };
    let keep_right = if bool {
      expr.op == BinaryOp::LogicalAnd
    } else {
      expr.op == BinaryOp::LogicalOr
    };
    if !param.could_have_side_effects() && (keep_right || param.is_bool()) {
      let dep = ConstDependency::new(
        param.range().0,
        param.range().1,
        format!(" {bool}").into(),
        None,
      );
      scanner.presentational_dependencies.push(Box::new(dep));
    } else {
      expr.left.visit_children_with(scanner);
    }
    if !keep_right {
      let dep = ConstDependency::new(
        expr.right.span().real_lo(),
        expr.right.span().hi().0,
        "0".into(),
        None,
      );
      scanner.presentational_dependencies.push(Box::new(dep));
    }
    Some(keep_right)
  } else {
    expr.visit_children_with(scanner);
    None
  }
  // else if expr.op == BinaryOp::NullishCoalescing {
  //   // TODO: support `??`
  //   expr.visit_children_with(scanner);
  //   None
  // }
}
