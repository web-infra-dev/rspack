use rspack_core::SpanExt;
use swc_core::common::Spanned;
use swc_core::ecma::ast::Lit;

use super::BasicEvaluatedExpression;

pub fn eval_lit_expr(expr: &Lit) -> Option<BasicEvaluatedExpression> {
  match expr {
    Lit::Str(str) => {
      let mut res = BasicEvaluatedExpression::new();
      res.set_range(str.span().real_lo(), str.span_hi().0);
      res.set_string(str.value.to_string());
      Some(res)
    }
    // TODO:
    _ => None,
  }
}
