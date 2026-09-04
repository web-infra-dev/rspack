use rspack_util::SpanExt;
use swc_next_ecma_ast::{Ast, Expr, ExprData, GetSpan};

use super::BasicEvaluatedExpression;

#[inline]
pub fn eval_lit_expr<'a>(ast: &Ast<'_>, expr: Expr) -> Option<BasicEvaluatedExpression<'a>> {
  let span = expr.span(ast);
  let mut result = BasicEvaluatedExpression::with_range(span.real_lo(), span.real_hi());
  match ast.expr_data(expr) {
    ExprData::StringLiteral(string) => {
      result.set_string(
        ast
          .get_wtf8(string.value(ast))
          .to_string_lossy()
          .into_owned(),
      );
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
