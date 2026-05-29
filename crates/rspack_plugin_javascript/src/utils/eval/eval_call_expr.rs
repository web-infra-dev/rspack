use swc_core::ecma::ast::{CallExpr, Callee, MemberProp};

use super::BasicEvaluatedExpression;
use crate::{
  parser_plugin::{
    CREATE_REQUIRE_SPECIFIER_TAG, CreateRequireSpecifierTagData, JavascriptParserPlugin,
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
        let is_create_require = parser
          .get_tag_data::<CreateRequireSpecifierTagData>(&ident.sym, CREATE_REQUIRE_SPECIFIER_TAG)
          .is_some();
        let evaluated = if is_create_require {
          ident.sym.call_hooks_name(parser, |parser, for_name| {
            drive.evaluate_call_expression(parser, for_name, expr)
          })
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
