use swc_core::ecma::ast::{CallExpr, MemberProp};

use super::BasicEvaluatedExpression;
use crate::{parser_plugin::JavascriptParserPlugin, visitors::JavascriptParser};

#[inline]
pub fn eval_call_expression<'a>(
  parser: &mut JavascriptParser,
  expr: &'a CallExpr,
) -> Option<BasicEvaluatedExpression<'a>> {
  if let Some(member) = expr.callee.as_expr().and_then(|expr| expr.as_member()) {
    if let MemberProp::Ident(ident) = &member.prop {
      let param = parser.evaluate_expression(&member.obj);
      parser.plugin_drive.clone().evaluate_call_expression_member(
        parser,
        ident.sym.as_str(),
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
