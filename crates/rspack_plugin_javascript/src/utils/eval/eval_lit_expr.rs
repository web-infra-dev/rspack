use rspack_util::SpanExt;
use swc_core::{
  common::Spanned,
  ecma::ast::{Lit, Str},
};

use super::BasicEvaluatedExpression;

#[inline]
pub fn eval_str(str: &Str) -> BasicEvaluatedExpression<'_> {
  let mut res = BasicEvaluatedExpression::with_range(str.span().real_lo(), str.span().real_hi());
  res.set_string(str.value.to_string_lossy().to_string());
  res
}

#[inline]
pub fn eval_number(num: &swc_core::ecma::ast::Number) -> BasicEvaluatedExpression<'_> {
  let mut res = BasicEvaluatedExpression::with_range(num.span().real_lo(), num.span().real_hi());
  res.set_number(num.value);
  res
}

#[inline]
pub fn eval_bool(bool: &swc_core::ecma::ast::Bool) -> BasicEvaluatedExpression<'_> {
  let mut res = BasicEvaluatedExpression::with_range(bool.span().real_lo(), bool.span().real_hi());
  res.set_bool(bool.value);
  res
}

#[inline]
pub fn eval_bigint(bigint: &swc_core::ecma::ast::BigInt) -> BasicEvaluatedExpression<'_> {
  let mut res =
    BasicEvaluatedExpression::with_range(bigint.span().real_lo(), bigint.span().real_hi());
  res.set_bigint((*bigint.value).clone());
  res
}

#[inline]
pub fn eval_lit_expr(expr: &Lit) -> Option<BasicEvaluatedExpression<'_>> {
  match expr {
    Lit::Str(str) => Some(eval_str(str)),
    Lit::Regex(regexp) => {
      let mut res =
        BasicEvaluatedExpression::with_range(regexp.span().real_lo(), regexp.span().real_hi());
      res.set_regexp(regexp.exp.to_string(), regexp.flags.to_string());
      Some(res)
    }
    Lit::Null(null) => {
      let mut res = BasicEvaluatedExpression::with_range(null.span.real_lo(), null.span.real_hi());
      res.set_null();
      Some(res)
    }
    Lit::Num(num) => Some(eval_number(num)),
    Lit::Bool(bool) => Some(eval_bool(bool)),
    Lit::BigInt(bigint) => Some(eval_bigint(bigint)),
    Lit::JSXText(_) => unreachable!(),
  }
}
