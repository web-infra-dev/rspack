use std::collections::VecDeque;

use swc_core::common::SyntaxContext;
use swc_core::ecma::ast::*;
use swc_core::ecma::atoms::JsWord;

pub fn extract_member_expression_chain(
  expression: &MemberExpr,
) -> VecDeque<(JsWord, SyntaxContext)> {
  let mut members: VecDeque<(JsWord, SyntaxContext)> = VecDeque::new();
  let mut expr = expression;

  loop {
    if let MemberProp::Computed(ComputedPropName {
      expr: box Expr::Lit(Lit::Str(ref val)),
      ..
    }) = expr.prop
    {
      members.push_front((val.value.clone(), val.span.ctxt));
    } else if let MemberProp::Ident(ref ident) = expr.prop {
      members.push_front((ident.sym.clone(), ident.span.ctxt));
    } else {
      break;
    }
    match expr.obj {
      box Expr::Member(ref member_expr) => {
        expr = member_expr;
      }
      box Expr::Ident(ref ident) => {
        members.push_front((ident.sym.clone(), ident.span.ctxt));
        break;
      }
      _ => break,
    }
  }
  members
}
