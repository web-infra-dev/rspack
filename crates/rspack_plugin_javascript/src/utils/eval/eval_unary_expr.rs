use rspack_util::SpanExt;
use swc_core::{
  common::Spanned,
  ecma::ast::{Lit, UnaryExpr, UnaryOp},
};

use super::BasicEvaluatedExpression;
use crate::{
  parser_plugin::JavascriptParserPlugin,
  visitors::{CallHooksName, JavascriptParser, RootName},
};

#[inline]
fn eval_typeof<'a>(
  parser: &mut JavascriptParser,
  expr: &'a UnaryExpr,
) -> Option<BasicEvaluatedExpression<'a>> {
  assert!(expr.op == UnaryOp::TypeOf);
  if let Some(ident) = expr.arg.as_ident()
    && let Some(res) = ident.sym.call_hooks_name(parser, |parser, for_name| {
      parser
        .plugin_drive
        .clone()
        .evaluate_typeof(parser, expr, for_name)
    })
  {
    return Some(res);
  } else if let Some(meta_prop) = expr.arg.as_meta_prop()
    && let Some(res) = meta_prop.get_root_name().and_then(|name| {
      name.call_hooks_name(parser, |parser, for_name| {
        parser
          .plugin_drive
          .clone()
          .evaluate_typeof(parser, expr, for_name)
      })
    })
  {
    return Some(res);
  } else if let Some(member_expr) = expr.arg.as_member()
    && let Some(res) = member_expr.call_hooks_name(parser, |parser, for_name| {
      parser
        .plugin_drive
        .clone()
        .evaluate_typeof(parser, expr, for_name)
    })
  {
    return Some(res);
  } else if let Some(chain_expr) = expr.arg.as_opt_chain()
    && let Some(res) = chain_expr.call_hooks_name(parser, |parser, for_name| {
      parser
        .plugin_drive
        .clone()
        .evaluate_typeof(parser, expr, for_name)
    })
  {
    return Some(res);
  } else if expr.arg.as_fn_expr().is_some() {
    let mut res = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.real_hi());
    res.set_string("function".to_string());
    return Some(res);
  }

  let arg = parser.evaluate_expression(&expr.arg);
  if arg.is_unknown() {
    let arg = &*expr.arg;
    if arg.as_fn_expr().is_some() || arg.as_class().is_some() {
      let mut res = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.real_hi());
      res.set_string("function".to_string());
      Some(res)
    } else if let Some(unary) = arg.as_unary()
      && matches!(unary.op, UnaryOp::Minus | UnaryOp::Plus)
      && let Some(lit) = unary.arg.as_lit()
      && matches!(lit, Lit::Num(_))
    {
      let mut res = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.real_hi());
      res.set_string("number".to_string());
      Some(res)
    } else {
      None
    }
  } else if arg.is_string() {
    let mut res = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.real_hi());
    res.set_string("string".to_string());
    Some(res)
  } else if arg.is_undefined() {
    let mut res = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.real_hi());
    res.set_string("undefined".to_string());
    Some(res)
  } else if arg.is_number() {
    let mut res = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.real_hi());
    res.set_string("number".to_string());
    Some(res)
  } else if arg.is_null() || arg.is_regexp() || arg.is_array() {
    let mut res = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.real_hi());
    res.set_string("object".to_string());
    Some(res)
  } else if arg.is_bool() {
    let mut res = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.real_hi());
    res.set_string("boolean".to_string());
    Some(res)
  } else if arg.is_bigint() {
    let mut res = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.real_hi());
    res.set_string("bigint".to_string());
    Some(res)
  } else {
    // TODO: `arg.is_wrapped()`...
    None
  }
}

#[inline]
pub fn eval_unary_expression<'a>(
  scanner: &mut JavascriptParser,
  expr: &'a UnaryExpr,
) -> Option<BasicEvaluatedExpression<'a>> {
  match expr.op {
    UnaryOp::TypeOf => eval_typeof(scanner, expr),
    UnaryOp::Bang => {
      let arg = scanner.evaluate_expression(&expr.arg);
      let boolean = arg.as_bool()?;
      let mut eval =
        BasicEvaluatedExpression::with_range(expr.span().real_lo(), expr.span().real_hi());
      eval.set_bool(!boolean);
      eval.set_side_effects(arg.could_have_side_effects());
      Some(eval)
    }
    UnaryOp::Tilde => {
      let arg = scanner.evaluate_expression(&expr.arg);
      let number = arg.as_int()?;
      let mut eval =
        BasicEvaluatedExpression::with_range(expr.span().real_lo(), expr.span().real_hi());
      eval.set_number(!number as f64);
      eval.set_side_effects(arg.could_have_side_effects());
      Some(eval)
    }
    UnaryOp::Minus | UnaryOp::Plus => {
      let arg = scanner.evaluate_expression(&expr.arg);
      let number = arg.as_number()?;
      let res = match &expr.op {
        UnaryOp::Minus => -number,
        UnaryOp::Plus => number,
        _ => unreachable!(),
      };
      let mut eval =
        BasicEvaluatedExpression::with_range(expr.span().real_lo(), expr.span().real_hi());
      eval.set_number(res);
      eval.set_side_effects(arg.could_have_side_effects());
      Some(eval)
    }
    _ => None,
  }
}
