use rspack_core::SpanExt;
use swc_core::common::Spanned;
use swc_core::ecma::ast::{Lit, UnaryExpr, UnaryOp};

use super::BasicEvaluatedExpression;
use crate::parser_plugin::JavascriptParserPlugin;
use crate::visitors::{CallHooksName, JavascriptParser};

#[inline]
fn eval_typeof(
  parser: &mut JavascriptParser,
  expr: &UnaryExpr,
) -> Option<BasicEvaluatedExpression> {
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
  }

  // TODO: if let `MetaProperty`, `MemberExpression` ...
  let arg = parser.evaluate_expression(&expr.arg);
  if arg.is_unknown() {
    let arg = expr.arg.unwrap_parens();
    if arg.as_fn_expr().is_some() || arg.as_class().is_some() {
      let mut res = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.hi.0);
      res.set_string("function".to_string());
      Some(res)
    } else if let Some(unary) = arg.as_unary()
      && matches!(unary.op, UnaryOp::Minus | UnaryOp::Plus)
      && let Some(lit) = unary.arg.as_lit()
      && matches!(lit, Lit::Num(_))
    {
      let mut res = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.hi.0);
      res.set_string("number".to_string());
      Some(res)
    } else {
      None
    }
  } else if arg.is_string() {
    let mut res = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.hi.0);
    res.set_string("string".to_string());
    Some(res)
  } else if arg.is_undefined() {
    let mut res = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.hi.0);
    res.set_string("undefined".to_string());
    Some(res)
  } else if arg.is_number() {
    let mut res = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.hi.0);
    res.set_string("number".to_string());
    Some(res)
  } else if arg.is_null() || arg.is_regexp() || arg.is_array() {
    let mut res = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.hi.0);
    res.set_string("object".to_string());
    Some(res)
  } else if arg.is_bool() {
    let mut res = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.hi.0);
    res.set_string("boolean".to_string());
    Some(res)
  } else if arg.is_bigint() {
    let mut res = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.hi.0);
    res.set_string("bigint".to_string());
    Some(res)
  } else {
    // TODO: `arg.is_wrapped()`...
    None
  }
}

#[inline]
pub fn eval_unary_expression(
  scanner: &mut JavascriptParser,
  expr: &UnaryExpr,
) -> Option<BasicEvaluatedExpression> {
  match expr.op {
    UnaryOp::TypeOf => eval_typeof(scanner, expr),
    UnaryOp::Bang => {
      let arg = scanner.evaluate_expression(&expr.arg);
      let Some(boolean) = arg.as_bool() else {
        return None;
      };
      let mut eval = BasicEvaluatedExpression::with_range(expr.span().real_lo(), expr.span_hi().0);
      eval.set_bool(!boolean);
      eval.set_side_effects(arg.could_have_side_effects());
      Some(eval)
    }
    _ => None,
  }
}
