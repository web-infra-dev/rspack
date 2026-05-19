use swc_core::ecma::ast::{CallExpr, Callee, MemberProp};

use super::BasicEvaluatedExpression;
use crate::{parser_plugin::JavascriptParserPlugin, visitors::JavascriptParser};

#[inline]
pub fn eval_call_expression<'a>(
  parser: &mut JavascriptParser,
  expr: &'a CallExpr,
) -> Option<BasicEvaluatedExpression<'a>> {
  let drive = parser.plugin_drive.clone();
  match &expr.callee {
    Callee::Expr(callee_expr) => {
      if let Some(ident) = callee_expr.as_ident()
        && let Some(evaluated) = drive.evaluate_call_expression(parser, ident.sym.as_str(), expr)
      {
        return Some(evaluated);
      }
      if let Some(member) = callee_expr.as_member()
        && let MemberProp::Ident(ident) = &member.prop
      {
        let param = parser.evaluate_expression(&member.obj);
        return drive.evaluate_call_expression_member(parser, ident.sym.as_str(), expr, param);
      }
      None
    }
    _ => None,
  }
}
