use rspack_core::SpanExt;
use swc_core::common::Spanned;
use swc_core::ecma::ast::{Lit, PropName, Str};

use super::BasicEvaluatedExpression;

#[inline]
fn eval_str(str: &Str) -> BasicEvaluatedExpression {
  let mut res = BasicEvaluatedExpression::with_range(str.span().real_lo(), str.span_hi().0);
  res.set_string(str.value.to_string());
  res
}

#[inline]
fn eval_number(num: &swc_core::ecma::ast::Number) -> BasicEvaluatedExpression {
  let mut res = BasicEvaluatedExpression::with_range(num.span().real_lo(), num.span_hi().0);
  res.set_number(num.value);
  res
}

#[inline]
fn eval_bool(bool: &swc_core::ecma::ast::Bool) -> BasicEvaluatedExpression {
  let mut res = BasicEvaluatedExpression::with_range(bool.span().real_lo(), bool.span_hi().0);
  res.set_bool(bool.value);
  res
}

#[inline]
fn eval_bigint(bigint: &swc_core::ecma::ast::BigInt) -> BasicEvaluatedExpression {
  let mut res = BasicEvaluatedExpression::with_range(bigint.span().real_lo(), bigint.span_hi().0);
  res.set_bigint((*bigint.value).clone());
  res
}

#[inline]
pub fn eval_lit_expr(expr: &Lit) -> Option<BasicEvaluatedExpression> {
  match expr {
    Lit::Str(str) => Some(eval_str(str)),
    Lit::Regex(regexp) => {
      let mut res =
        BasicEvaluatedExpression::with_range(regexp.span().real_lo(), regexp.span_hi().0);
      res.set_regexp(regexp.exp.to_string(), regexp.flags.to_string());
      Some(res)
    }
    Lit::Null(null) => {
      let mut res = BasicEvaluatedExpression::with_range(null.span.real_lo(), null.span.hi().0);
      res.set_null();
      Some(res)
    }
    Lit::Num(num) => Some(eval_number(num)),
    Lit::Bool(bool) => Some(eval_bool(bool)),
    Lit::BigInt(bigint) => Some(eval_bigint(bigint)),
    Lit::JSXText(_) => unreachable!(),
  }
}

#[inline]
pub fn eval_prop_name(prop_name: &PropName) -> Option<BasicEvaluatedExpression> {
  match prop_name {
    PropName::Str(str) => Some(eval_str(str)),
    PropName::Num(num) => Some(eval_number(num)),
    PropName::BigInt(bigint) => Some(eval_bigint(bigint)),
    // TODO:
    PropName::Ident(_) => None,
    PropName::Computed(_) => None,
  }
}
