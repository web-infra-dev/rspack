use swc_atoms::{js_word, JsWord};
use swc_common::Mark;
use swc_ecma_ast::{CallExpr, Callee, Expr, ExprOrSpread, Id, Ident, Lit};

pub fn get_first_string_lit_arg(e: &CallExpr) -> Option<JsWord> {
  // we check the length at the begin of [is_require_literal_expr]
  e.args.first().and_then(|arg| match arg {
    ExprOrSpread {
      spread: None,
      expr: box Expr::Lit(Lit::Str(str)),
    } => Some(str.value.clone()),
    _ => None,
  })
  //   match e.args.first().expect("this should never happen") {
  //     ExprOrSpread { spread: None, expr } => match &**expr {
  //       Expr::Lit(Lit::Str(str)) => !str.value.is_empty(),
  //       _ => false,
  //     },
  //     _ => false,
  //   }
}

pub fn is_require_literal_expr(e: &CallExpr, unresolved_mark: Mark) -> bool {
  if e.args.len() == 1 {
    let first_arg = get_first_string_lit_arg(e);

    first_arg.is_some()
      && match &e.callee {
        ident @ Callee::Expr(box Expr::Ident(Ident {
          sym: js_word!("require"),
          ..
        })) => ident
          .as_expr()
          .and_then(|expr| expr.as_ident())
          .map(|ident| ident.span.ctxt.outer() == unresolved_mark)
          .unwrap_or(false),
        _ => false,
      }
  } else {
    false
  }
}

pub fn get_dynamic_import_string_literal(e: &CallExpr) -> Option<JsWord> {
  if e.args.len() == 1 && matches!(&e.callee, Callee::Import(_)) {
    get_first_string_lit_arg(e)
  } else {
    None
  }
}
