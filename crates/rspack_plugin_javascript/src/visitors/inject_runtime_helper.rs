use super::module_variables::WEBPACK_PUBLIC_PATH;
use crate::visitors::module_variables::WEBPACK_HASH;
use hashbrown::HashSet;
use rspack_core::runtime_globals;
use swc_core::common::{Mark, DUMMY_SP};
use swc_core::ecma::ast::*;
use swc_core::ecma::transforms::base::helpers::HELPERS;
use swc_core::ecma::utils::ExprFactory;
use swc_core::ecma::visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith};

pub fn inject_runtime_helper(
  unresolved_mark: Mark,
  runtime_requirements: &mut HashSet<String>,
) -> impl Fold + '_ {
  let helper_mark = HELPERS.with(|helper| helper.mark());
  as_folder(InjectRuntimeHelper {
    helper_mark,
    unresolved_mark,
    runtime_requirements,
  })
}

struct InjectRuntimeHelper<'a> {
  helper_mark: Mark,
  unresolved_mark: Mark,
  runtime_requirements: &'a mut HashSet<String>,
}

impl<'a> VisitMut for InjectRuntimeHelper<'a> {
  noop_visit_mut_type!();

  fn visit_mut_ident(&mut self, n: &mut Ident) {
    if n.span.has_mark(self.unresolved_mark) {
      if WEBPACK_HASH.eq(&n.sym) {
        self
          .runtime_requirements
          .insert(runtime_globals::GET_FULL_HASH.to_string());
      }
      if WEBPACK_PUBLIC_PATH.eq(&n.sym) {
        self
          .runtime_requirements
          .insert(runtime_globals::PUBLIC_PATH.to_string());
      }
    }
  }

  fn visit_mut_call_expr(&mut self, n: &mut CallExpr) {
    n.visit_mut_children_with(self);
    if let Some(box Expr::Ident(ident)) = n.callee.as_expr() {
      // must have helper mark
      if !ident.span.has_mark(self.helper_mark) {
        return;
      }

      let word = ident.sym.as_ref();
      if matches!(
        word,
        "_interop_require_default" | "_interop_require_wildcard"
      ) {
        self
          .runtime_requirements
          .insert(runtime_globals::INTEROP_REQUIRE.to_string());
        // TODO try with ast.parse(r#"self["__rspack_runtime__"].interopRequire"#)
        n.callee = MemberExpr {
          span: DUMMY_SP,
          obj: Box::new(Expr::Ident(Ident::new(
            runtime_globals::REQUIRE.into(),
            DUMMY_SP,
          ))),
          prop: MemberProp::Ident(Ident::new("interopRequire".into(), DUMMY_SP)),
        }
        .as_callee();
        n.args.visit_mut_children_with(self);
        return;
      }

      if matches!(word, "_export_star") {
        self
          .runtime_requirements
          .insert(runtime_globals::EXPORT_STAR.to_string());
        // TODO try with ast.parse(r#"self["__rspack_runtime__"].exportStar"#)
        n.callee = MemberExpr {
          span: DUMMY_SP,
          obj: Box::new(Expr::Ident(Ident::new(
            runtime_globals::REQUIRE.into(),
            DUMMY_SP,
          ))),
          prop: MemberProp::Ident(Ident::new("exportStar".into(), DUMMY_SP)),
        }
        .as_callee();
        n.args.visit_mut_children_with(self);
        return;
      }

      // have some unhandled helper
      debug_assert!(false, "have unhandled helper: word = {}", word);
    }
  }
}
