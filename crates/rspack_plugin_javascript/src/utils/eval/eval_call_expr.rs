use swc_core::ecma::ast::{CallExpr, Callee, MemberProp};

use super::BasicEvaluatedExpression;
use crate::{
  parser_plugin::{
    CREATE_REQUIRE_EVALUATED_TAG, JavascriptParserPlugin, is_create_require_specifier,
  },
  visitors::{CallHooksName, JavascriptParser},
};

#[inline]
pub fn eval_call_expression<'a>(
  parser: &mut JavascriptParser,
  expr: &'a CallExpr,
) -> Option<BasicEvaluatedExpression<'a>> {
  let drive = parser.plugin_drive.clone();
  match &expr.callee {
    Callee::Expr(callee_expr) => {
      if let Some(ident) = callee_expr.as_ident() {
        let is_create_require = parser.javascript_options.is_create_require_enabled()
          && is_create_require_specifier(parser, &ident.sym);
        let evaluated = if is_create_require {
          ident.sym.call_hooks_name(parser, |parser, for_name| {
            drive.evaluate_call_expression(parser, for_name, expr)
          })
        } else if parser.javascript_options.is_create_require_enabled() {
          let evaluated = parser.evaluate_expression(callee_expr);
          if evaluated.is_identifier() && evaluated.identifier() == CREATE_REQUIRE_EVALUATED_TAG {
            drive.evaluate_call_expression(parser, CREATE_REQUIRE_EVALUATED_TAG, expr)
          } else {
            drive.evaluate_call_expression(parser, ident.sym.as_str(), expr)
          }
        } else {
          drive.evaluate_call_expression(parser, ident.sym.as_str(), expr)
        };
        if evaluated.is_some() {
          return evaluated;
        }
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
