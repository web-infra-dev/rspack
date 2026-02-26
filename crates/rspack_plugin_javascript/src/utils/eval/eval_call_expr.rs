use swc_experimental_ecma_ast::{CallExpr, MemberProp};

use super::BasicEvaluatedExpression;
use crate::{parser_plugin::JavascriptParserPlugin, visitors::JavascriptParser};

#[inline]
pub fn eval_call_expression(
  parser: &mut JavascriptParser,
  expr: CallExpr,
) -> Option<BasicEvaluatedExpression> {
  if let Some(member) = expr
    .callee(&parser.ast)
    .as_expr()
    .and_then(|expr| expr.as_member())
  {
    if let MemberProp::Ident(ident) = member.prop(&parser.ast) {
      let param = parser.evaluate_expression(member.obj(&parser.ast));
      parser.plugin_drive.clone().evaluate_call_expression_member(
        parser,
        ident.sym(&parser.ast),
        expr,
        param.clone(),
      )
    } else {
      None
    }
  } else {
    None
  }
}
