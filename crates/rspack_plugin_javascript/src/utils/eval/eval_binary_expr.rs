use rspack_core::{DependencyLocation, SpanExt};
use swc_core::ecma::ast::{BinExpr, BinaryOp};

use crate::{utils::eval::BasicEvaluatedExpression, visitors::JavascriptParser};

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

fn is_always_different(a: Option<bool>, b: Option<bool>) -> bool {
  match (a, b) {
    (Some(a), Some(b)) => a != b,
    _ => false,
  }
}

/// `eql` is `true` for `===` and `false` for `!==`
fn handle_strict_equality_comparison(
  eql: bool,
  expr: &BinExpr,
  scanner: &mut JavascriptParser,
) -> Option<BasicEvaluatedExpression> {
  assert!(expr.op == BinaryOp::EqEqEq || expr.op == BinaryOp::NotEqEq);
  let left = scanner.evaluate_expression(&expr.left);
  let right = scanner.evaluate_expression(&expr.right);
  let mut res = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.hi().0);
  let left_const = left.is_compile_time_value();
  let right_const = right.is_compile_time_value();

  let common = |mut res: BasicEvaluatedExpression| {
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
fn handle_abstract_equality_comparison(
  eql: bool,
  expr: &BinExpr,
  scanner: &mut JavascriptParser,
) -> Option<BasicEvaluatedExpression> {
  assert!(expr.op == BinaryOp::EqEq || expr.op == BinaryOp::NotEq);
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

fn handle_logical_or(
  expr: &BinExpr,
  scanner: &mut JavascriptParser,
) -> Option<BasicEvaluatedExpression> {
  let mut res = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.hi().0);
  let left = scanner.evaluate_expression(&expr.left);

  match left.as_bool() {
    Some(true) => {
      // true || unknown = true
      res.set_bool(true);
      res.set_side_effects(left.could_have_side_effects());
      Some(res)
    }
    Some(false) => {
      let right = scanner.evaluate_expression(&expr.right);
      // false || unknown = unknown
      right.as_bool().map(|b| {
        // false || right = right
        res.set_bool(b);
        res.set_side_effects(left.could_have_side_effects() || right.could_have_side_effects());
        res
      })
    }
    None => {
      let right = scanner.evaluate_expression(&expr.right);
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

fn handle_logical_and(
  expr: &BinExpr,
  scanner: &mut JavascriptParser,
) -> Option<BasicEvaluatedExpression> {
  let mut res = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.hi().0);
  let left = scanner.evaluate_expression(&expr.left);
  match left.as_bool() {
    Some(true) => {
      // true && unknown = unknown
      let mut right = scanner.evaluate_expression(&expr.right);
      if left.could_have_side_effects() {
        right.set_side_effects(true)
      }
      right.set_range(expr.span.real_lo(), expr.span.hi.0);
      Some(right)
    }
    Some(false) => {
      // false && any = false
      res.set_bool(false);
      res.set_side_effects(left.could_have_side_effects());
      Some(res)
    }
    None => {
      let right = scanner.evaluate_expression(&expr.right);
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

fn handle_add(expr: &BinExpr, scanner: &mut JavascriptParser) -> Option<BasicEvaluatedExpression> {
  assert_eq!(expr.op, BinaryOp::Add);
  let left = scanner.evaluate_expression(&expr.left);
  let right = scanner.evaluate_expression(&expr.right);
  let mut res = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.hi.0);
  if left.could_have_side_effects() || right.could_have_side_effects() {
    res.set_side_effects(true)
  }
  if left.is_string() {
    if right.is_string() {
      res.set_string(format!("{}{}", left.string(), right.string()));
    } else if right.is_number() {
      res.set_string(format!("{}{}", left.string(), right.number()));
    } else if right.is_wrapped()
      && let Some(prefix) = right.prefix()
      && prefix.is_string()
    {
      let (start, end) = join_locations(left.range.as_ref(), prefix.range.as_ref());
      let mut left_prefix = BasicEvaluatedExpression::with_range(start, end);
      left_prefix.set_string(format!("{}{}", left.string(), prefix.string()));
      res.set_wrapped(
        Some(left_prefix),
        right.postfix.map(|postfix| *postfix),
        right
          .wrapped_inner_expressions
          .expect("wrapped_inner_expressions must be exists under wrapped"),
      )
    } else if right.is_wrapped() {
      res.set_wrapped(
        Some(left),
        right.postfix.map(|postfix| *postfix),
        right
          .wrapped_inner_expressions
          .expect("wrapped_inner_expressions must be exists under wrapped"),
      );
    } else {
      res.set_wrapped(Some(left), None, vec![right])
    }
  } else if left.is_number() {
    if right.is_string() {
      res.set_string(format!("{}{}", left.number(), right.string()));
    } else if right.is_number() {
      res.set_number(left.number() + right.number())
    } else {
      return None;
    }
  } else if left.is_bigint() {
    // TODO: handle `left.is_bigint`
    return None;
  } else if left.is_wrapped() {
    if let Some(postfix) = left.postfix {
      if postfix.is_string() && right.is_string() {
        let range = join_locations(postfix.range.as_ref(), right.range.as_ref());
        let mut right_postfix = BasicEvaluatedExpression::with_range(range.0, range.1);
        right_postfix.set_string(format!("{}{}", postfix.string(), right.string()));
        res.set_wrapped(
          left.prefix.map(|prefix| *prefix),
          Some(right_postfix),
          left
            .wrapped_inner_expressions
            .expect("wrapped_inner_expressions must be exists under wrapped"),
        )
      } else if postfix.is_string() && right.is_number() {
        let range = join_locations(postfix.range.as_ref(), right.range.as_ref());
        let mut right_postfix = BasicEvaluatedExpression::with_range(range.0, range.1);
        right_postfix.set_string(format!("{}{}", postfix.string(), right.number()));
        res.set_wrapped(
          left.prefix.map(|prefix| *prefix),
          Some(right_postfix),
          left
            .wrapped_inner_expressions
            .expect("wrapped_inner_expressions must be exists under wrapped"),
        )
      }
    } else if right.is_string() {
      res.set_wrapped(
        left.prefix.map(|prefix| *prefix),
        Some(right),
        left
          .wrapped_inner_expressions
          .expect("wrapped_inner_expressions must be exists under wrapped"),
      );
    } else if right.is_number() {
      let range = right.range();
      let mut postfix = BasicEvaluatedExpression::with_range(range.0, range.1);
      postfix.set_string(right.number().to_string());
      res.set_wrapped(
        left.prefix.map(|prefix| *prefix),
        Some(postfix),
        left
          .wrapped_inner_expressions
          .expect("wrapped_inner_expressions must be exists under wrapped"),
      )
    } else if right.is_wrapped() {
      let inner_expressions = if let Some(mut left_inner_expression) =
        left.wrapped_inner_expressions
        && let Some(mut right_inner_expression) = right.wrapped_inner_expressions
      {
        if let Some(postfix) = left.postfix {
          left_inner_expression.push(*postfix);
        }
        if let Some(prefix) = right.prefix {
          left_inner_expression.push(*prefix);
        }
        left_inner_expression.append(&mut right_inner_expression);
        left_inner_expression
      } else {
        vec![]
      };
      res.set_wrapped(
        left.prefix.map(|prefix| *prefix),
        right.postfix.map(|postfix| *postfix),
        inner_expressions,
      );
    } else {
      let inner_expressions =
        if let Some(mut left_inner_expression) = left.wrapped_inner_expressions {
          if let Some(postfix) = left.postfix {
            left_inner_expression.push(*postfix);
          }
          left_inner_expression.push(right);
          left_inner_expression
        } else {
          vec![]
        };
      res.set_wrapped(left.prefix.map(|prefix| *prefix), None, inner_expressions)
    }
  } else if right.is_string() {
    res.set_wrapped(None, Some(right), vec![left]);
  } else if right.is_wrapped() {
    let mut inner_expressions = if let Some(right_prefix) = right.prefix {
      vec![left, *right_prefix]
    } else {
      vec![left]
    };
    if let Some(mut right_inner_expressions) = right.wrapped_inner_expressions {
      inner_expressions.append(&mut right_inner_expressions)
    }
    res.set_wrapped(
      None,
      right.postfix.map(|postfix| *postfix),
      inner_expressions,
    );
  } else {
    return None;
  }

  Some(res)
}

pub fn eval_binary_expression(
  scanner: &mut JavascriptParser,
  expr: &BinExpr,
) -> Option<BasicEvaluatedExpression> {
  match expr.op {
    BinaryOp::EqEq => handle_abstract_equality_comparison(true, expr, scanner),
    BinaryOp::NotEq => handle_abstract_equality_comparison(false, expr, scanner),
    BinaryOp::EqEqEq => handle_strict_equality_comparison(true, expr, scanner),
    BinaryOp::NotEqEq => handle_strict_equality_comparison(false, expr, scanner),
    BinaryOp::LogicalAnd => handle_logical_and(expr, scanner),
    BinaryOp::LogicalOr => handle_logical_or(expr, scanner),
    BinaryOp::Add => handle_add(expr, scanner),
    _ => None,
  }
}

fn join_locations(
  start: Option<&DependencyLocation>,
  end: Option<&DependencyLocation>,
) -> (u32, u32) {
  match (start, end) {
    (None, None) => unreachable!("invalid range"),
    (None, Some(end)) => (end.start(), end.end()),
    (Some(start), None) => (start.start(), start.end()),
    (Some(start), Some(end)) => join_ranges(
      Some((start.start(), start.end())),
      Some((end.start(), end.end())),
    ),
  }
}

fn join_ranges(start: Option<(u32, u32)>, end: Option<(u32, u32)>) -> (u32, u32) {
  match (start, end) {
    (None, None) => unreachable!("invalid range"),
    (None, Some(end)) => end,
    (Some(start), None) => start,
    (Some(start), Some(end)) => {
      assert!(start.0 <= end.1);
      (start.0, end.1)
    }
  }
}
