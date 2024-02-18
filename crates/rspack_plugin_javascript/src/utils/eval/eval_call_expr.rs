use swc_core::ecma::ast::{CallExpr, MemberProp};

use super::BasicEvaluatedExpression;
use crate::{parser_plugin::JavascriptParserPlugin, visitors::JavascriptParser};

pub fn eval_call_expression(
  parser: &mut JavascriptParser,
  expr: &CallExpr,
) -> Option<BasicEvaluatedExpression> {
  if let Some(member) = expr.callee.as_expr().and_then(|expr| expr.as_member()) {
    if let MemberProp::Ident(ident) = &member.prop {
      let param = parser.evaluate_expression(&member.obj);
      parser.plugin_drive.clone().evaluate_call_expression_member(
        parser,
        ident.sym.as_str(),
        expr,
        &param,
      )
    } else {
      None
    }
  } else {
    None
  }
}
