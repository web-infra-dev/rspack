use rspack_core::ConstDependency;
use rspack_core::SpanExt;
use swc_core::common::Spanned;
use swc_core::ecma::ast::{BinExpr, BinaryOp};

use crate::visitors::JavascriptParser;

pub fn is_logic_op(op: BinaryOp) -> bool {
  matches!(
    op,
    BinaryOp::LogicalAnd | BinaryOp::LogicalOr | BinaryOp::NullishCoalescing
  )
}

pub fn expression_logic_operator(scanner: &mut JavascriptParser, expr: &BinExpr) -> Option<bool> {
  if expr.op == BinaryOp::LogicalAnd || expr.op == BinaryOp::LogicalOr {
    let param = scanner.evaluate_expression(&expr.left);
    let boolean = param.as_bool();
    let Some(boolean) = boolean else {
      return None;
    };
    let keep_right = if boolean {
      expr.op == BinaryOp::LogicalAnd
    } else {
      expr.op == BinaryOp::LogicalOr
    };
    if !param.could_have_side_effects() && (keep_right || param.is_bool()) {
      scanner
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          param.range().0,
          param.range().1 - 1,
          format!(" {boolean}").into(),
          None,
        )));
    } else {
      scanner.walk_expression(&expr.left);
    }

    if !keep_right {
      scanner
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          expr.right.span().real_lo(),
          expr.right.span().hi().0 - 1,
          "0".into(),
          None,
        )));
    }
    Some(keep_right)
  } else if expr.op == BinaryOp::NullishCoalescing {
    let param = scanner.evaluate_expression(&expr.left);
    if let Some(keep_right) = param.as_nullish() {
      if !param.could_have_side_effects() && keep_right {
        scanner
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(
            param.range().0,
            param.range().1 - 1,
            " null".into(),
            None,
          )));
      } else {
        scanner
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(
            expr.right.span().real_lo(),
            expr.right.span().hi().0 - 1,
            "0".into(),
            None,
          )));
        scanner.walk_expression(&expr.left);
      }
      Some(keep_right)
    } else {
      None
    }
  } else {
    unreachable!()
  }
}
