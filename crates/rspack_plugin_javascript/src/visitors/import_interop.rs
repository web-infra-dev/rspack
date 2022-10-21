use crate::runtime::RSPACK_RUNTIME;
use swc_common::DUMMY_SP;
use swc_ecma_ast::*;
use swc_ecma_utils::ExprFactory;
use swc_ecma_visit::{as_folder, noop_visit_mut_type, Fold, VisitMut};

pub fn import_interop() -> impl Fold {
  as_folder(ImportInterop {})
}

struct ImportInterop {}

impl VisitMut for ImportInterop {
  noop_visit_mut_type!();

  fn visit_mut_call_expr(&mut self, n: &mut CallExpr) {
    if let Some(box Expr::Ident(ident)) = n.callee.as_expr() {
      let word = ident.sym.to_string();
      if matches!(
        word.as_str(),
        "_interop_require_default" | "_interop_require_wildcard"
      ) {
        // TODO try with ast.parse(r#"self["__rspack_runtime__"].interopRequire"#)
        n.callee = MemberExpr {
          span: DUMMY_SP,
          obj: Box::new(Expr::Ident(Ident::new(RSPACK_RUNTIME.into(), DUMMY_SP))),
          prop: MemberProp::Ident(Ident::new("interopRequire".into(), DUMMY_SP)),
        }
        .as_callee()
      }
    }
  }
}
