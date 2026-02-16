use rspack_util::SpanExt;
use swc_experimental_ecma_ast::{PropName, Spanned};

use crate::{
  utils::eval::{BasicEvaluatedExpression, eval_bigint, eval_number, eval_str},
  visitors::JavascriptParser,
};

#[inline]
pub fn eval_prop_name(
  parser: &mut JavascriptParser,
  prop_name: PropName,
) -> BasicEvaluatedExpression {
  match prop_name {
    PropName::Str(str) => eval_str(parser, str),
    PropName::Num(num) => eval_number(parser, num),
    PropName::BigInt(bigint) => eval_bigint(parser, bigint),
    PropName::Ident(ident) => {
      let mut evaluated = BasicEvaluatedExpression::with_range(
        ident.span(&parser.ast).real_lo(),
        ident.span(&parser.ast).real_hi(),
      );
      evaluated.set_string(parser.ast.get_utf8(ident.sym(&parser.ast)).to_string());
      evaluated
    }
    PropName::Computed(computed) => parser.evaluate_expression(computed.expr(&parser.ast)),
  }
}
