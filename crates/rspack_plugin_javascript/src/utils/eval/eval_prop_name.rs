use rspack_util::SpanExt;
use swc_core::{common::Spanned, ecma::ast::PropName};

use crate::{
  utils::eval::{BasicEvaluatedExpression, eval_bigint, eval_number, eval_str},
  visitors::JavascriptParser,
};

#[inline]
pub fn eval_prop_name<'a>(
  parser: &mut JavascriptParser,
  prop_name: &'a PropName,
) -> BasicEvaluatedExpression<'a> {
  match prop_name {
    PropName::Str(str) => eval_str(str),
    PropName::Num(num) => eval_number(num),
    PropName::BigInt(bigint) => eval_bigint(bigint),
    PropName::Ident(ident) => {
      let mut evaluated =
        BasicEvaluatedExpression::with_range(ident.span().real_lo(), ident.span().real_hi());
      evaluated.set_string(ident.sym.to_string());
      evaluated
    }
    PropName::Computed(computed) => parser.evaluate_expression(&computed.expr),
  }
}
