use std::borrow::Cow;

use rspack_util::SpanExt;
use swc_next_allocator::wtf8::Wtf8;
use swc_next_ecma_ast::{Ast, Expr, ExprData, GetSpan};

use super::BasicEvaluatedExpression;

#[inline]
fn wtf8_to_string_lossy(value: &Wtf8) -> Cow<'_, str> {
  match std::str::from_utf8(value.as_bytes()) {
    Ok(value) => Cow::Borrowed(value),
    Err(_) => value.to_string_lossy(),
  }
}

#[inline]
pub fn eval_lit_expr<'a>(ast: &'a Ast<'_>, expr: Expr) -> Option<BasicEvaluatedExpression<'a>> {
  let span = expr.span(ast);
  let mut result = BasicEvaluatedExpression::with_range(span.real_lo(), span.real_hi());
  match ast.expr_data(expr) {
    ExprData::StringLiteral(string) => {
      result.set_string(wtf8_to_string_lossy(ast.get_wtf8(string.value(ast))));
    }
    ExprData::RegExpLiteral(regexp) => result.set_regexp(
      ast.get_utf8(regexp.pattern(ast)).to_string(),
      ast.get_utf8(regexp.flags(ast)).to_string(),
    ),
    ExprData::NullLiteral(_) => result.set_null(),
    ExprData::NumericLiteral(number) => result.set_number(number.value(ast)),
    ExprData::BooleanLiteral(boolean) => result.set_bool(boolean.value(ast)),
    ExprData::BigIntLiteral(bigint) => {
      result.set_bigint(ast.get_utf8(bigint.raw(ast)).parse().ok()?);
    }
    _ => return None,
  }
  Some(result)
}
