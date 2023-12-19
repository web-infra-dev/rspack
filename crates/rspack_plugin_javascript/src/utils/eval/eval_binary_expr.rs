use rspack_core::SpanExt;
use swc_core::ecma::ast::{BinExpr, BinaryOp};

use crate::utils::eval::BasicEvaluatedExpression;
use crate::visitors::common_js_import_dependency_scanner::CommonJsImportDependencyScanner;

fn handle_template_string_compare(
  left: &BasicEvaluatedExpression,
  right: &BasicEvaluatedExpression,
  mut res: BasicEvaluatedExpression,
  eql: bool,
) -> Option<BasicEvaluatedExpression> {
  let get_prefix = |parts: &Vec<BasicEvaluatedExpression>| {
    let mut value = vec![];
    for p in parts {
      if let Some(s) = p.as_string() {
        value.push(s);
      } else {
        break;
      }
    }
    value.concat()
  };
  let get_suffix = |parts: &Vec<BasicEvaluatedExpression>| {
    let mut value = vec![];
    for p in parts.iter().rev() {
      if let Some(s) = p.as_string() {
        value.push(s);
      } else {
        break;
      }
    }
    value.concat()
  };

  let prefix_res = {
    let left_prefix = get_prefix(left.parts());
    let right_prefix = get_prefix(right.parts());
    let len_prefix = usize::min(left_prefix.len(), right_prefix.len());
    len_prefix > 0 && left_prefix[0..len_prefix] != right_prefix[0..len_prefix]
  };
  if prefix_res {
    res.set_bool(!eql);
    res.set_side_effects(left.could_have_side_effects() || right.could_have_side_effects());
    return Some(res);
  }

  let suffix_res = {
    let left_suffix = get_suffix(left.parts());
    let right_suffix = get_suffix(right.parts());
    let len_suffix = usize::min(left_suffix.len(), right_suffix.len());
    len_suffix > 0
      && left_suffix[left_suffix.len() - len_suffix..]
        != right_suffix[right_suffix.len() - len_suffix..]
  };
  if suffix_res {
    res.set_bool(!eql);
    res.set_side_effects(left.could_have_side_effects() || right.could_have_side_effects());
    return Some(res);
  }

  None
}

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
  } else if left.is_template_string() && right.is_template_string() {
    handle_template_string_compare(&left, &right, res, eql)
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
  } else if left.is_template_string() && right.is_template_string() {
    handle_template_string_compare(&left, &right, res, eql)
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
