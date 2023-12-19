use rspack_core::SpanExt;
use swc_core::ecma::ast::{BinExpr, BinaryOp};

use crate::utils::eval::BasicEvaluatedExpression;
use crate::visitors::common_js_import_dependency_scanner::CommonJsImportDependencyScanner;

/// `eql` is `true` for `===` and `false` for `!==`
fn handle_strict_equality_comparison<'a>(
  eql: bool,
  expr: &'a BinExpr,
  scanner: &'a CommonJsImportDependencyScanner<'a>,
) -> Option<BasicEvaluatedExpression> {
  assert!(expr.op == BinaryOp::EqEqEq || expr.op == BinaryOp::NotEqEq);
  let left = scanner.evaluate_expression(&expr.left);
  let right = scanner.evaluate_expression(&expr.right);
  let mut res = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.hi().0);
  let left_const = left.is_compile_time_value();
  let right_const = right.is_compile_time_value();

  if left_const && right_const {
    res.set_bool(eql == left.compare_compile_time_value(&right));
    res.set_side_effects(left.could_have_side_effects() || right.could_have_side_effects());
    Some(res)
  } else if left.is_array() && right.is_array() {
    res.set_bool(!eql);
    res.set_side_effects(left.could_have_side_effects() || right.could_have_side_effects());
    Some(res)
  } else {
    None
  }
}
/// `eql` is `true` for `==` and `false` for `!=`
fn handle_abstract_equality_comparison<'a>(
  eql: bool,
  expr: &'a BinExpr,
  scanner: &'a CommonJsImportDependencyScanner<'a>,
) -> Option<BasicEvaluatedExpression> {
  let left = scanner.evaluate_expression(&expr.left);
  let right = scanner.evaluate_expression(&expr.right);
  let mut res = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.hi().0);

  let left_const = left.is_compile_time_value();
  let right_const = right.is_compile_time_value();

  if left_const && right_const {
    res.set_bool(eql == left.compare_compile_time_value(&right));
    res.set_side_effects(left.could_have_side_effects() || right.could_have_side_effects());
    Some(res)
  } else if left.is_array() && right.is_array() {
    res.set_bool(!eql);
    res.set_side_effects(left.could_have_side_effects() || right.could_have_side_effects());
    Some(res)
  } else {
    None
  }
}

pub fn eval_binary_expression<'a>(
  scanner: &'a CommonJsImportDependencyScanner<'a>,
  expr: &'a BinExpr,
) -> Option<BasicEvaluatedExpression> {
  match expr.op {
    BinaryOp::EqEq => handle_abstract_equality_comparison(true, expr, scanner),
    BinaryOp::NotEq => handle_abstract_equality_comparison(false, expr, scanner),
    BinaryOp::EqEqEq => handle_strict_equality_comparison(true, expr, scanner),
    BinaryOp::NotEqEq => handle_strict_equality_comparison(false, expr, scanner),
    _ => None,
  }
}
