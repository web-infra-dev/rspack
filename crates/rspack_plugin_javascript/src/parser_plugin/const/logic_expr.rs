use rspack_core::ConstDependency;
use rspack_util::SpanExt;
use swc_experimental_ecma_ast::{BinExpr, BinaryOp, Spanned};

use crate::visitors::JavascriptParser;

pub fn is_logic_op(op: BinaryOp) -> bool {
  matches!(
    op,
    BinaryOp::LogicalAnd | BinaryOp::LogicalOr | BinaryOp::NullishCoalescing
  )
}

pub fn expression_logic_operator(scanner: &mut JavascriptParser, expr: BinExpr) -> Option<bool> {
  if expr.op(&scanner.ast) == BinaryOp::LogicalAnd || expr.op(&scanner.ast) == BinaryOp::LogicalOr {
    let param = scanner.evaluate_expression(expr.left(&scanner.ast));
    let boolean = param.as_bool();
    let boolean = boolean?;
    let keep_right = if boolean {
      expr.op(&scanner.ast) == BinaryOp::LogicalAnd
    } else {
      expr.op(&scanner.ast) == BinaryOp::LogicalOr
    };
    if !param.could_have_side_effects() && (keep_right || param.is_bool()) {
      scanner.add_presentational_dependency(Box::new(ConstDependency::new(
        param.range().into(),
        format!(" {boolean}").into(),
      )));
    } else {
      scanner.walk_expression(expr.left(&scanner.ast));
    }

    if !keep_right {
      scanner.add_presentational_dependency(Box::new(ConstDependency::new(
        (
          expr.right(&scanner.ast).span(&scanner.ast).real_lo(),
          expr.right(&scanner.ast).span(&scanner.ast).real_hi(),
        )
          .into(),
        "0".into(),
      )));
    }
    Some(keep_right)
  } else if expr.op(&scanner.ast) == BinaryOp::NullishCoalescing {
    let param = scanner.evaluate_expression(expr.left(&scanner.ast));
    if let Some(keep_right) = param.as_nullish() {
      if !param.could_have_side_effects() && keep_right {
        scanner.add_presentational_dependency(Box::new(ConstDependency::new(
          param.range().into(),
          " null".into(),
        )));
      } else {
        scanner.add_presentational_dependency(Box::new(ConstDependency::new(
          (
            expr.right(&scanner.ast).span(&scanner.ast).real_lo(),
            expr.right(&scanner.ast).span(&scanner.ast).real_hi(),
          )
            .into(),
          "0".into(),
        )));
        scanner.walk_expression(expr.left(&scanner.ast));
      }
      Some(keep_right)
    } else {
      None
    }
  } else {
    unreachable!()
  }
}
