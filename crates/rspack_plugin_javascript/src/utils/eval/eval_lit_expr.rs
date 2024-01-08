use rspack_core::SpanExt;
use swc_core::common::Spanned;
use swc_core::ecma::ast::Lit;

use super::BasicEvaluatedExpression;

pub fn eval_lit_expr(expr: &Lit) -> Option<BasicEvaluatedExpression> {
  match expr {
    Lit::Str(str) => {
      let mut res = BasicEvaluatedExpression::with_range(str.span().real_lo(), str.span_hi().0);
      res.set_string(str.value.to_string());
      Some(res)
    }
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
    // TODO:
    _ => None,
  }
}
