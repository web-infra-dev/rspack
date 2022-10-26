use crate::runtime::RSPACK_RUNTIME;
use swc_common::{Mark, DUMMY_SP};
use swc_ecma_ast::*;
use swc_ecma_transforms::helpers::HELPERS;
use swc_ecma_utils::ExprFactory;
use swc_ecma_visit::{as_folder, noop_visit_mut_type, Fold, VisitMut};

pub fn inject_runtime_helper() -> impl Fold {
  let helper_mark = HELPERS.with(|helper| helper.mark());
  as_folder(InjectRuntimeHelper { helper_mark })
}

struct InjectRuntimeHelper {
  helper_mark: Mark,
}

impl VisitMut for InjectRuntimeHelper {
  noop_visit_mut_type!();

  fn visit_mut_call_expr(&mut self, n: &mut CallExpr) {
    if let Some(box Expr::Ident(ident)) = n.callee.as_expr() {
      // must have helper mark
      // if !ident.span.has_mark(self.helper_mark) {
      //   return;
      // }

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
        .as_callee();
        return;
      }

      if matches!(word.as_str(), "_export_star") {
        // TODO try with ast.parse(r#"self["__rspack_runtime__"].exportStar"#)
        n.callee = MemberExpr {
          span: DUMMY_SP,
          obj: Box::new(Expr::Ident(Ident::new(RSPACK_RUNTIME.into(), DUMMY_SP))),
          prop: MemberProp::Ident(Ident::new("exportStar".into(), DUMMY_SP)),
        }
        .as_callee();
        return;
      }

      // have some unhandled helper
      // debug_assert!(false, "have unhandled helper: word = {}", word);
    }
  }
}
