use rspack_util::SpanExt;
use swc_experimental_ecma_ast::{BigInt, Bool, Lit, Number, Regex, Str};

use super::BasicEvaluatedExpression;

#[inline]
pub fn eval_str<'a>(str: &'a Str<'a>) -> BasicEvaluatedExpression<'a> {
  let mut res = BasicEvaluatedExpression::with_range(str.span.real_lo(), str.span.real_hi());
  res.set_string(str.value.as_wtf8().to_string_lossy().to_string());
  res
}

#[inline]
pub fn eval_number<'a>(num: &'a Number<'a>) -> BasicEvaluatedExpression<'a> {
  let mut res = BasicEvaluatedExpression::with_range(num.span.real_lo(), num.span.real_hi());
  res.set_number(num.value);
  res
}

#[inline]
pub fn eval_bool(bool: &Bool) -> BasicEvaluatedExpression<'_> {
  let mut res = BasicEvaluatedExpression::with_range(bool.span.real_lo(), bool.span.real_hi());
  res.set_bool(bool.value);
  res
}

#[inline]
pub fn eval_bigint<'a>(bigint: &'a BigInt<'a>) -> BasicEvaluatedExpression<'a> {
  let mut res = BasicEvaluatedExpression::with_range(bigint.span.real_lo(), bigint.span.real_hi());
  res.set_bigint(bigint.to_bigint());
  res
}

#[inline]
fn eval_regex<'a>(regexp: &'a Regex<'a>) -> BasicEvaluatedExpression<'a> {
  let mut res = BasicEvaluatedExpression::with_range(regexp.span.real_lo(), regexp.span.real_hi());
  res.set_regexp(regexp.exp.to_string(), regexp.flags.to_string());
  res
}

#[inline]
pub fn eval_lit_expr<'a>(expr: &'a Lit<'a>) -> Option<BasicEvaluatedExpression<'a>> {
  match expr {
    Lit::Str(str) => Some(eval_str(str)),
    Lit::Regex(regexp) => Some(eval_regex(regexp)),
    Lit::Null(null) => {
      let mut res = BasicEvaluatedExpression::with_range(null.span.real_lo(), null.span.real_hi());
      res.set_null();
      Some(res)
    }
    Lit::Num(num) => Some(eval_number(num)),
    Lit::Bool(bool) => Some(eval_bool(bool)),
    Lit::BigInt(bigint) => Some(eval_bigint(bigint)),
  }
}
