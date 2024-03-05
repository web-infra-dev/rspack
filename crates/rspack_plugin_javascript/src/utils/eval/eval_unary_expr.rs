use rspack_core::SpanExt;
use swc_core::common::Spanned;
use swc_core::ecma::ast::{UnaryExpr, UnaryOp};

use super::BasicEvaluatedExpression;
use crate::parser_plugin::JavascriptParserPlugin;
use crate::visitors::JavascriptParser;

fn eval_typeof(
  parser: &mut JavascriptParser,
  expr: &UnaryExpr,
) -> Option<BasicEvaluatedExpression> {
  assert!(expr.op == UnaryOp::TypeOf);
  if let Some(ident) = expr.arg.as_ident()
    && /* FIXME: should use call hooks for name */ let res = parser.plugin_drive.clone().evaluate_typeof(
      parser,
      ident,
      expr.span.real_lo(),
      expr.span.hi().0,
    )
    && res.is_some()
  {
    return res;
  }

  // TODO: if let `MetaProperty`, `MemberExpression` ...
  let arg = parser.evaluate_expression(&expr.arg);
  if arg.is_unknown() {
    None
  } else if arg.is_string() {
    let mut res = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.hi.0);
    res.set_string("string".to_string());
    Some(res)
  } else if arg.is_undefined() {
    let mut res = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.hi.0);
    res.set_string("undefined".to_string());
    Some(res)
  } else {
    // TODO: `arg.is_wrapped()`...
    None
  }
}

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
