use rspack_core::DependencyRange;
use rspack_util::SpanExt;
use swc_next_ecma_ast::{
  BinaryExpression, BinaryOperator, GetSpan, LogicalExpression, LogicalOperator,
};

use crate::{
  parser_plugin::JavascriptParserPlugin, utils::eval::BasicEvaluatedExpression,
  visitors::JavascriptParser,
};

#[inline]
fn handle_template_string_compare<'parser>(
  left: &BasicEvaluatedExpression,
  right: &BasicEvaluatedExpression,
  mut res: BasicEvaluatedExpression<'parser>,
  eql: bool,
) -> Option<BasicEvaluatedExpression<'parser>> {
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

#[inline]
fn is_always_different(a: Option<bool>, b: Option<bool>) -> bool {
  match (a, b) {
    (Some(a), Some(b)) => a != b,
    _ => false,
  }
}

/// `eql` is `true` for `===` and `false` for `!==`
#[inline]
fn handle_strict_equality_comparison<'parser>(
  eql: bool,
  left: BasicEvaluatedExpression<'parser>,
  expr: BinaryExpression,
  scanner: &mut JavascriptParser<'parser>,
) -> Option<BasicEvaluatedExpression<'parser>> {
  let ast = scanner.ast.ast;
  assert!(matches!(
    expr.operator(ast),
    BinaryOperator::StrictEqual | BinaryOperator::StrictNotEqual
  ));
  let right = scanner.evaluate_expression(expr.right(ast));
  let mut res =
    BasicEvaluatedExpression::with_range(expr.span(ast).real_lo(), expr.span(ast).real_hi());
  let left_const = left.is_compile_time_value();
  let right_const = right.is_compile_time_value();

  let common = |mut res: BasicEvaluatedExpression<'parser>| {
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
#[inline(always)]
fn handle_abstract_equality_comparison<'parser>(
  eql: bool,
  left: BasicEvaluatedExpression<'parser>,
  expr: BinaryExpression,
  scanner: &mut JavascriptParser<'parser>,
) -> Option<BasicEvaluatedExpression<'parser>> {
  let ast = scanner.ast.ast;
  assert!(matches!(
    expr.operator(ast),
    BinaryOperator::Equal | BinaryOperator::NotEqual
  ));
  let right = scanner.evaluate_expression(expr.right(ast));
  let mut res =
    BasicEvaluatedExpression::with_range(expr.span(ast).real_lo(), expr.span(ast).real_hi());

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

#[inline(always)]
fn handle_nullish_coalescing<'parser>(
  left: BasicEvaluatedExpression<'parser>,
  expr: LogicalExpression,
  scanner: &mut JavascriptParser<'parser>,
) -> Option<BasicEvaluatedExpression<'parser>> {
  let ast = scanner.ast.ast;
  let left_nullish = left.as_nullish();
  match left_nullish {
    Some(true) => {
      let mut right = scanner.evaluate_expression(expr.right(ast));
      if left.could_have_side_effects() {
        right.set_side_effects(true)
      }
      right.set_range(expr.span(ast).real_lo(), expr.span(ast).real_hi());
      Some(right)
    }
    Some(false) => {
      let mut res = left;
      res.set_range(expr.span(ast).real_lo(), expr.span(ast).real_hi());
      Some(res)
    }
    _ => None,
  }
}

#[inline(always)]
fn handle_logical_or<'parser>(
  mut left: BasicEvaluatedExpression<'parser>,
  expr: LogicalExpression,
  scanner: &mut JavascriptParser<'parser>,
) -> Option<BasicEvaluatedExpression<'parser>> {
  let ast = scanner.ast.ast;
  let left_bool = left.as_bool();
  match left_bool {
    Some(true) => {
      let mut res = left;
      res.set_range(expr.span(ast).real_lo(), expr.span(ast).real_hi());
      Some(res)
    }
    Some(false) => {
      let mut right = scanner.evaluate_expression(expr.right(ast));
      if left.could_have_side_effects() {
        right.set_side_effects(true)
      }
      right.set_range(expr.span(ast).real_lo(), expr.span(ast).real_hi());
      Some(right)
    }
    None => {
      let right = scanner.evaluate_expression(expr.right(ast));
      let right_bool = right.as_bool();
      if right_bool == Some(true) {
        let mut res =
          BasicEvaluatedExpression::with_range(expr.span(ast).real_lo(), expr.span(ast).real_hi());
        res.set_truthy();
        Some(res)
      } else if left.is_dependency() {
        if right_bool == Some(false) {
          left.set_range(expr.span(ast).real_lo(), expr.span(ast).real_hi());
          Some(left)
        } else if right.is_dependency() {
          let mut res = BasicEvaluatedExpression::with_range(
            expr.span(ast).real_lo(),
            expr.span(ast).real_hi(),
          );
          res.set_dependency(left.into_dependency().or(right.into_dependency()));
          Some(res)
        } else {
          None
        }
      } else {
        None
      }
    }
  }
}

#[inline(always)]
fn handle_logical_and<'parser>(
  mut left: BasicEvaluatedExpression<'parser>,
  expr: LogicalExpression,
  scanner: &mut JavascriptParser<'parser>,
) -> Option<BasicEvaluatedExpression<'parser>> {
  let ast = scanner.ast.ast;
  let left_bool = left.as_bool();
  match left_bool {
    Some(true) => {
      let mut right = scanner.evaluate_expression(expr.right(ast));
      if left.could_have_side_effects() {
        right.set_side_effects(true)
      }
      right.set_range(expr.span(ast).real_lo(), expr.span(ast).real_hi());
      Some(right)
    }
    Some(false) => {
      let mut res = left;
      res.set_range(expr.span(ast).real_lo(), expr.span(ast).real_hi());
      Some(res)
    }
    None => {
      let right = scanner.evaluate_expression(expr.right(ast));
      let right_bool = right.as_bool();
      if right_bool == Some(false) {
        let mut res =
          BasicEvaluatedExpression::with_range(expr.span(ast).real_lo(), expr.span(ast).real_hi());
        res.set_falsy();
        Some(res)
      } else if left.is_dependency() {
        if right_bool == Some(true) {
          left.set_range(expr.span(ast).real_lo(), expr.span(ast).real_hi());
          Some(left)
        } else if right.is_dependency() {
          let mut res = BasicEvaluatedExpression::with_range(
            expr.span(ast).real_lo(),
            expr.span(ast).real_hi(),
          );
          res.set_dependency(left.into_dependency().and(right.into_dependency()));
          Some(res)
        } else {
          None
        }
      } else {
        None
      }
    }
  }
}

#[inline(always)]
fn handle_add<'parser>(
  left: BasicEvaluatedExpression<'parser>,
  expr: BinaryExpression,
  scanner: &mut JavascriptParser<'parser>,
) -> Option<BasicEvaluatedExpression<'parser>> {
  let ast = scanner.ast.ast;
  assert_eq!(expr.operator(ast), BinaryOperator::Add);
  let right = scanner.evaluate_expression(expr.right(ast));
  let mut res =
    BasicEvaluatedExpression::with_range(expr.span(ast).real_lo(), expr.span(ast).real_hi());
  if left.could_have_side_effects() || right.could_have_side_effects() {
    res.set_side_effects(true)
  }
  if left.is_string() {
    if right.is_string() {
      res.set_string(format!("{}{}", left.string(), right.string()));
    } else if right.is_number() {
      res.set_string(format!("{}{}", left.string(), right.number()));
    } else if right.is_wrapped() {
      let (right_prefix, right_postfix, right_inner_expressions) =
        right.into_wrapped().expect("right should be wrapped");
      if let Some(prefix) = right_prefix.as_ref()
        && prefix.is_string()
      {
        let (start, end) = join_locations(left.range_ref(), prefix.range_ref());
        let mut left_prefix = BasicEvaluatedExpression::with_range(start, end);
        left_prefix.set_string(format!("{}{}", left.string(), prefix.string()));
        res.set_wrapped(Some(left_prefix), right_postfix, right_inner_expressions)
      } else {
        res.set_wrapped(Some(left), right_postfix, right_inner_expressions);
      }
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
    let had_side_effects = res.could_have_side_effects();
    if let (Some(l), Some(r)) = (left.bigint(), right.bigint()) {
      res.set_bigint(l.clone() + r.clone());
      if had_side_effects {
        res.set_side_effects(true);
      }
      return Some(res);
    }
    return None;
  } else if left.is_wrapped() {
    let (mut left_prefix, mut left_postfix, mut left_inner_expressions) =
      left.into_wrapped().expect("left should be wrapped");
    if let Some(postfix) = left_postfix.as_ref()
      && postfix.is_string()
      && right.is_string()
    {
      let range = join_locations(postfix.range_ref(), right.range_ref());
      let mut right_postfix = BasicEvaluatedExpression::with_range(range.0, range.1);
      right_postfix.set_string(format!("{}{}", postfix.string(), right.string()));
      res.set_wrapped(
        left_prefix.take(),
        Some(right_postfix),
        std::mem::take(&mut left_inner_expressions),
      )
    } else if let Some(postfix) = left_postfix.as_ref()
      && postfix.is_string()
      && right.is_number()
    {
      let range = join_locations(postfix.range_ref(), right.range_ref());
      let mut right_postfix = BasicEvaluatedExpression::with_range(range.0, range.1);
      right_postfix.set_string(format!("{}{}", postfix.string(), right.number()));
      res.set_wrapped(
        left_prefix.take(),
        Some(right_postfix),
        std::mem::take(&mut left_inner_expressions),
      )
    } else if right.is_string() {
      res.set_wrapped(
        left_prefix.take(),
        Some(right),
        std::mem::take(&mut left_inner_expressions),
      );
    } else if right.is_number() {
      let range = right.range();
      let mut postfix = BasicEvaluatedExpression::with_range(range.0, range.1);
      postfix.set_string(right.number().to_string());
      res.set_wrapped(
        left_prefix.take(),
        Some(postfix),
        std::mem::take(&mut left_inner_expressions),
      )
    } else if right.is_wrapped() {
      let (right_prefix, right_postfix, mut right_inner_expression) =
        right.into_wrapped().expect("right should be wrapped");
      let mut inner_expressions = std::mem::take(&mut left_inner_expressions);
      if let Some(postfix) = left_postfix.take() {
        inner_expressions.push(postfix);
      }
      if let Some(prefix) = right_prefix {
        inner_expressions.push(prefix);
      }
      inner_expressions.append(&mut right_inner_expression);
      res.set_wrapped(left_prefix.take(), right_postfix, inner_expressions);
    } else {
      let mut inner_expressions = std::mem::take(&mut left_inner_expressions);
      if let Some(postfix) = left_postfix.take() {
        inner_expressions.push(postfix);
      }
      inner_expressions.push(right);
      res.set_wrapped(left_prefix.take(), None, inner_expressions)
    }
  } else if right.is_string() {
    res.set_wrapped(None, Some(right), vec![left]);
  } else if right.is_wrapped() {
    let (right_prefix, right_postfix, mut right_inner_expressions) =
      right.into_wrapped().expect("right should be wrapped");
    let mut inner_expressions = if let Some(right_prefix) = right_prefix {
      vec![left, right_prefix]
    } else {
      vec![left]
    };
    inner_expressions.append(&mut right_inner_expressions);
    res.set_wrapped(None, right_postfix, inner_expressions);
  } else {
    return None;
  }

  Some(res)
}

#[inline(always)]
pub fn handle_const_operation<'parser>(
  left: BasicEvaluatedExpression<'parser>,
  expr: BinaryExpression,
  scanner: &mut JavascriptParser<'parser>,
) -> Option<BasicEvaluatedExpression<'parser>> {
  let ast = scanner.ast.ast;
  if !left.is_compile_time_value() {
    return None;
  }
  let right = scanner.evaluate_expression(expr.right(ast));
  if !right.is_compile_time_value() {
    return None;
  }

  let had_side_effects = left.could_have_side_effects() || right.could_have_side_effects();
  let mut res =
    BasicEvaluatedExpression::with_range(expr.span(ast).real_lo(), expr.span(ast).real_hi());

  match expr.operator(ast) {
    BinaryOperator::Subtract
    | BinaryOperator::Multiply
    | BinaryOperator::Divide
    | BinaryOperator::Modulo
    | BinaryOperator::Exponent => {
      if let Some(left_number) = left.as_number()
        && let Some(right_number) = right.as_number()
      {
        res.set_number(match expr.operator(ast) {
          BinaryOperator::Subtract => left_number - right_number,
          BinaryOperator::Multiply => left_number * right_number,
          BinaryOperator::Divide => left_number / right_number,
          BinaryOperator::Modulo => left_number % right_number,
          BinaryOperator::Exponent => left_number.powf(right_number),
          _ => unreachable!(),
        });
        if had_side_effects {
          res.set_side_effects(true);
        }
        Some(res)
      } else {
        None
      }
    }
    BinaryOperator::LeftShift | BinaryOperator::RightShift | BinaryOperator::UnsignedRightShift => {
      if let Some(left_int) = left.as_int() {
        let right_int = right.as_int()?;
        let shift_bits = (right_int as u32) & 31;
        let result = match expr.operator(ast) {
          BinaryOperator::LeftShift => (left_int << shift_bits) as f64,
          BinaryOperator::RightShift => (left_int >> shift_bits) as f64,
          BinaryOperator::UnsignedRightShift => (left_int as u32).wrapping_shr(shift_bits) as f64,
          _ => unreachable!(),
        };
        res.set_number(result);
        if had_side_effects {
          res.set_side_effects(true);
        }
        Some(res)
      } else {
        None
      }
    }
    BinaryOperator::BitwiseAnd | BinaryOperator::BitwiseXor | BinaryOperator::BitwiseOr => {
      if let Some(left_number) = left.as_int()
        && let Some(right_number) = right.as_int()
      {
        res.set_number(match expr.operator(ast) {
          BinaryOperator::BitwiseAnd => left_number & right_number,
          BinaryOperator::BitwiseXor => left_number ^ right_number,
          BinaryOperator::BitwiseOr => left_number | right_number,
          _ => unreachable!(),
        } as f64);
        if had_side_effects {
          res.set_side_effects(true);
        }
        Some(res)
      } else {
        None
      }
    }
    BinaryOperator::LessThan
    | BinaryOperator::GreaterThan
    | BinaryOperator::LessThanOrEqual
    | BinaryOperator::GreaterThanOrEqual => {
      if left.is_string() && right.is_string() {
        let left_str = left.string();
        let right_str = right.string();
        res.set_bool(match expr.operator(ast) {
          BinaryOperator::LessThan => left_str < right_str,
          BinaryOperator::LessThanOrEqual => left_str <= right_str,
          BinaryOperator::GreaterThan => left_str > right_str,
          BinaryOperator::GreaterThanOrEqual => left_str >= right_str,
          _ => unreachable!(),
        });
        Some(res)
      } else if let Some(left_number) = left.as_number()
        && let Some(right_number) = right.as_number()
      {
        res.set_bool(match expr.operator(ast) {
          BinaryOperator::LessThan => left_number < right_number,
          BinaryOperator::LessThanOrEqual => left_number <= right_number,
          BinaryOperator::GreaterThan => left_number > right_number,
          BinaryOperator::GreaterThanOrEqual => left_number >= right_number,
          _ => unreachable!(),
        });
        Some(res)
      } else {
        None
      }
    }
    _ => None,
  }
}

pub fn eval_binary_expression<'parser>(
  scanner: &mut JavascriptParser<'parser>,
  expr: BinaryExpression,
) -> Option<BasicEvaluatedExpression<'parser>> {
  let ast = scanner.ast.ast;
  let mut stack = vec![expr];
  let mut left_expression = expr.left(ast);
  while let Some(bin) = left_expression.as_binary_expression(ast) {
    stack.push(bin);
    left_expression = bin.left(ast);
  }
  let mut evaluated = None;
  while let Some(expr) = stack.pop() {
    let left = evaluated.unwrap_or_else(|| scanner.evaluate_expression(expr.left(ast)));
    let drive = scanner.plugin_drive.clone();
    evaluated = drive
      .evaluate_binary_expression(scanner, expr, &left)
      .or_else(|| match expr.operator(ast) {
        BinaryOperator::Equal => handle_abstract_equality_comparison(true, left, expr, scanner),
        BinaryOperator::NotEqual => handle_abstract_equality_comparison(false, left, expr, scanner),
        BinaryOperator::StrictEqual => handle_strict_equality_comparison(true, left, expr, scanner),
        BinaryOperator::StrictNotEqual => {
          handle_strict_equality_comparison(false, left, expr, scanner)
        }
        BinaryOperator::Add => handle_add(left, expr, scanner),
        _ => handle_const_operation(left, expr, scanner),
      })
      .or_else(|| {
        Some(BasicEvaluatedExpression::with_range(
          expr.span(ast).real_lo(),
          expr.span(ast).real_hi(),
        ))
      });
  }
  evaluated
}

pub fn eval_logical_expression<'parser>(
  scanner: &mut JavascriptParser<'parser>,
  expr: LogicalExpression,
) -> Option<BasicEvaluatedExpression<'parser>> {
  let ast = scanner.ast.ast;
  let mut stack = vec![expr];
  let mut left_expression = expr.left(ast);
  while let Some(logical) = left_expression.as_logical_expression(ast) {
    stack.push(logical);
    left_expression = logical.left(ast);
  }
  let mut evaluated = None;
  while let Some(expr) = stack.pop() {
    let left = evaluated.unwrap_or_else(|| scanner.evaluate_expression(expr.left(ast)));
    evaluated = match expr.operator(ast) {
      LogicalOperator::And => handle_logical_and(left, expr, scanner),
      LogicalOperator::Or => handle_logical_or(left, expr, scanner),
      LogicalOperator::NullishCoalescing => handle_nullish_coalescing(left, expr, scanner),
    }
    .or_else(|| {
      Some(BasicEvaluatedExpression::with_range(
        expr.span(ast).real_lo(),
        expr.span(ast).real_hi(),
      ))
    });
  }
  evaluated
}

fn join_locations(start: Option<&DependencyRange>, end: Option<&DependencyRange>) -> (u32, u32) {
  match (start, end) {
    (None, None) => unreachable!("invalid range"),
    (None, Some(end)) => (end.start, end.end),
    (Some(start), None) => (start.start, start.end),
    (Some(start), Some(end)) => {
      join_ranges(Some((start.start, start.end)), Some((end.start, end.end)))
    }
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
