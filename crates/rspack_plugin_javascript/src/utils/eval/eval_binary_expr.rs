use std::borrow::Cow;

use rspack_core::SpanExt;
use swc_core::ecma::ast::{BinExpr, BinaryOp};

use crate::{
  parser_plugin::JavaScriptParserPluginDrive, utils::eval::BasicEvaluatedExpression,
  visitors::JavascriptParser,
};

fn handle_template_string_compare<'ast>(
  left: &BasicEvaluatedExpression<'ast>,
  right: &BasicEvaluatedExpression<'ast>,
  mut res: BasicEvaluatedExpression<'ast>,
  eql: bool,
) -> Option<BasicEvaluatedExpression<'ast>> {
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

fn is_always_different(a: Option<bool>, b: Option<bool>) -> bool {
  match (a, b) {
    (Some(a), Some(b)) => a != b,
    _ => false,
  }
}

/// `eql` is `true` for `===` and `false` for `!==`
fn handle_strict_equality_comparison<'ast, 'parser>(
  eql: bool,
  expr: &'ast BinExpr,
  scanner: &mut JavascriptParser<'parser>,
  plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
) -> Option<BasicEvaluatedExpression<'ast>> {
  assert!(expr.op == BinaryOp::EqEqEq || expr.op == BinaryOp::NotEqEq);
  let left = scanner.evaluate_expression(&expr.left, plugin_drive);
  let right = scanner.evaluate_expression(&expr.right, plugin_drive);
  let mut res = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.hi().0);
  let left_const = left.is_compile_time_value();
  let right_const = right.is_compile_time_value();

  let common = |mut res: BasicEvaluatedExpression<'ast>| {
    res.set_bool(!eql);
    res.set_side_effects(left.could_have_side_effects() || right.could_have_side_effects());
    Some(res)
  };

  if left_const && right_const {
    res.set_bool(eql == left.compare_compile_time_value(&right));
    res.set_side_effects(left.could_have_side_effects() || right.could_have_side_effects());
    Some(res)
  } else if left.is_array() && right.is_array() {
    common(res)
  } else if left.is_template_string() && right.is_template_string() {
    handle_template_string_compare(&left, &right, res, eql)
  } else if is_always_different(left.as_bool(), right.as_bool())
    || is_always_different(left.as_nullish(), right.as_nullish())
  {
    common(res)
  } else {
    let left_primitive = left.is_primitive_type();
    let right_primitive = right.is_primitive_type();
    if left_primitive == Some(false) && (left_const || right_primitive == Some(true))
      || (right_primitive == Some(false) && (right_const || left_primitive == Some(true)))
    {
      common(res)
    } else {
      None
    }
  }
}

/// `eql` is `true` for `==` and `false` for `!=`
fn handle_abstract_equality_comparison<'ast, 'parser>(
  eql: bool,
  expr: &'ast BinExpr,
  scanner: &mut JavascriptParser<'parser>,
  plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
) -> Option<BasicEvaluatedExpression<'ast>> {
  assert!(expr.op == BinaryOp::EqEq || expr.op == BinaryOp::NotEq);
  let left = scanner.evaluate_expression(&expr.left, plugin_drive);
  let right = scanner.evaluate_expression(&expr.right, plugin_drive);
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

fn handle_logical_or<'ast, 'parser>(
  expr: &'ast BinExpr,
  scanner: &mut JavascriptParser<'parser>,
  plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
) -> Option<BasicEvaluatedExpression<'ast>> {
  let mut res = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.hi().0);
  let left = scanner.evaluate_expression(&expr.left, plugin_drive);

  match left.as_bool() {
    Some(true) => {
      // true || unknown = true
      res.set_bool(true);
      res.set_side_effects(left.could_have_side_effects());
      Some(res)
    }
    Some(false) => {
      let right = scanner.evaluate_expression(&expr.right, plugin_drive);
      // false || unknown = unknown
      right.as_bool().map(|b| {
        // false || right = right
        res.set_bool(b);
        res.set_side_effects(left.could_have_side_effects() || right.could_have_side_effects());
        res
      })
    }
    None => {
      let right = scanner.evaluate_expression(&expr.right, plugin_drive);
      match right.as_bool() {
        // unknown || true = true
        Some(true) => {
          res.set_bool(true);
          res.set_side_effects(left.could_have_side_effects() || right.could_have_side_effects());
          Some(res)
        }
        // unknown || false/unknown = unknown
        _ => None,
      }
    }
  }
}

fn handle_logical_and<'ast, 'parser>(
  expr: &'ast BinExpr,
  scanner: &mut JavascriptParser<'parser>,
  plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
) -> Option<BasicEvaluatedExpression<'ast>> {
  let mut res = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.hi().0);

  let left = scanner.evaluate_expression(&expr.left, plugin_drive);

  match left.as_bool() {
    Some(true) => {
      let right = scanner.evaluate_expression(&expr.right, plugin_drive);
      // true && unknown = unknown
      right.as_bool().map(|b| {
        // true && right = right
        res.set_bool(b);
        res.set_side_effects(left.could_have_side_effects() || right.could_have_side_effects());
        res
      })
    }
    Some(false) => {
      // false && any = false
      res.set_bool(false);
      res.set_side_effects(left.could_have_side_effects());
      Some(res)
    }
    None => {
      let right = scanner.evaluate_expression(&expr.right, plugin_drive);
      match right.as_bool() {
        // unknown && false = false
        Some(false) => {
          res.set_bool(false);
          res.set_side_effects(left.could_have_side_effects() || right.could_have_side_effects());
          Some(res)
        }
        // unknown && true/unknown = unknown
        _ => None,
      }
    }
  }
}

fn handle_add<'ast, 'parser>(
  expr: &'ast BinExpr,
  scanner: &mut JavascriptParser<'parser>,
  plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
) -> Option<BasicEvaluatedExpression<'ast>> {
  assert_eq!(expr.op, BinaryOp::Add);
  let left = scanner.evaluate_expression(&expr.left, plugin_drive);
  let right = scanner.evaluate_expression(&expr.right, plugin_drive);
  let mut res = BasicEvaluatedExpression::new();
  if left.is_string() && right.is_string() {
    res.set_string(Cow::Owned(format!("{}{}", left.string(), right.string())));
    return Some(res);
    // TODO: right.is_number....
  }
  // TODO: left.is_number....
  None
}

pub fn eval_binary_expression<'ast, 'parser>(
  scanner: &mut JavascriptParser<'parser>,
  expr: &'ast BinExpr,
  plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
) -> Option<BasicEvaluatedExpression<'ast>> {
  match expr.op {
    BinaryOp::EqEq => handle_abstract_equality_comparison(true, expr, scanner, plugin_drive),
    BinaryOp::NotEq => handle_abstract_equality_comparison(false, expr, scanner, plugin_drive),
    BinaryOp::EqEqEq => handle_strict_equality_comparison(true, expr, scanner, plugin_drive),
    BinaryOp::NotEqEq => handle_strict_equality_comparison(false, expr, scanner, plugin_drive),
    BinaryOp::LogicalAnd => handle_logical_and(expr, scanner, plugin_drive),
    BinaryOp::LogicalOr => handle_logical_or(expr, scanner, plugin_drive),
    BinaryOp::Add => handle_add(expr, scanner, plugin_drive),
    _ => None,
  }
}
