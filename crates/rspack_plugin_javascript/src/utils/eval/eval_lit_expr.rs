use rspack_util::SpanExt;
use swc_experimental_ecma_ast::{BigInt, Bool, Lit, Number, Spanned, Str};

use super::BasicEvaluatedExpression;
use crate::visitors::JavascriptParser;

#[inline]
pub fn eval_str(scanner: &mut JavascriptParser, str: Str) -> BasicEvaluatedExpression {
  let mut res = BasicEvaluatedExpression::with_range(
    str.span(&scanner.ast).real_lo(),
    str.span(&scanner.ast).real_hi(),
  );
  res.set_string(
    scanner
      .ast
      .get_wtf8(str.value(&scanner.ast))
      .to_string_lossy()
      .to_string(),
  );
  res
}

#[inline]
pub fn eval_number(scanner: &mut JavascriptParser, num: Number) -> BasicEvaluatedExpression {
  let mut res = BasicEvaluatedExpression::with_range(
    num.span(&scanner.ast).real_lo(),
    num.span(&scanner.ast).real_hi(),
  );
  res.set_number(num.value(&scanner.ast));
  res
}

#[inline]
pub fn eval_bool(scanner: &mut JavascriptParser, bool: Bool) -> BasicEvaluatedExpression {
  let mut res = BasicEvaluatedExpression::with_range(
    bool.span(&scanner.ast).real_lo(),
    bool.span(&scanner.ast).real_hi(),
  );
  res.set_bool(bool.value(&scanner.ast));
  res
}

#[inline]
pub fn eval_bigint(scanner: &mut JavascriptParser, bigint: BigInt) -> BasicEvaluatedExpression {
  let mut res = BasicEvaluatedExpression::with_range(
    bigint.span(&scanner.ast).real_lo(),
    bigint.span(&scanner.ast).real_hi(),
  );
  res.set_bigint(scanner.ast.get_big_int(bigint.value(&scanner.ast)).clone());
  res
}

#[inline]
pub fn eval_lit_expr(
  scanner: &mut JavascriptParser,
  expr: Lit,
) -> Option<BasicEvaluatedExpression> {
  match expr {
    Lit::Str(str) => Some(eval_str(scanner, str)),
    Lit::Regex(regexp) => {
      let mut res = BasicEvaluatedExpression::with_range(
        regexp.span(&scanner.ast).real_lo(),
        regexp.span(&scanner.ast).real_hi(),
      );
      res.set_regexp(
        scanner.ast.get_utf8(regexp.exp(&scanner.ast)).to_string(),
        scanner.ast.get_utf8(regexp.flags(&scanner.ast)).to_string(),
      );
      Some(res)
    }
    Lit::Null(null) => {
      let mut res = BasicEvaluatedExpression::with_range(
        null.span(&scanner.ast).real_lo(),
        null.span(&scanner.ast).real_hi(),
      );
      res.set_null();
      Some(res)
    }
    Lit::Num(num) => Some(eval_number(scanner, num)),
    Lit::Bool(bool) => Some(eval_bool(scanner, bool)),
    Lit::BigInt(bigint) => Some(eval_bigint(scanner, bigint)),
  }
}
