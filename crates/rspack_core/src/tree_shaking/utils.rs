use swc_core::common::Mark;
use swc_core::ecma::ast::{CallExpr, Callee, Expr, ExprOrSpread, Ident, Lit};
use swc_core::ecma::atoms::{js_word, JsWord};

pub fn get_first_string_lit_arg(e: &CallExpr) -> Option<JsWord> {
  // we check the length at the begin of [is_require_literal_expr]
  e.args.first().and_then(|arg| match arg {
    ExprOrSpread {
      spread: None,
      expr: box Expr::Lit(Lit::Str(str)),
    } => Some(str.value.clone()),
    _ => None,
  })
}

pub fn get_require_literal(e: &CallExpr, unresolved_mark: Mark) -> Option<JsWord> {
  if e.args.len() == 1 {
    if match &e.callee {
      ident @ Callee::Expr(box Expr::Ident(Ident {
        sym: js_word!("require"),
        ..
      })) => {
        // dbg!(&ident);
        ident
          .as_expr()
          .and_then(|expr| expr.as_ident())
          .map(|ident| ident.span.ctxt.outer() == unresolved_mark)
          .unwrap_or(false)
      }
      _ => false,
    } {
      get_first_string_lit_arg(e)
    } else {
      None
    }
  } else {
    None
  }
}

pub fn get_dynamic_import_string_literal(e: &CallExpr) -> Option<JsWord> {
  if e.args.len() == 1 && matches!(&e.callee, Callee::Import(_)) {
    get_first_string_lit_arg(e)
  } else {
    None
  }
}

/// # Panic
/// when module_identifier is not a pattern of xxx|xxxxxxxxxxxxxxxxxx
pub fn get_path_of_module_identifier<T: AsRef<str>>(path: T) -> String {
  let t = path.as_ref();
  let (_, path) = t.split_once('|').expect("Expect have `|` delimiter ");
  path.to_string()
}
