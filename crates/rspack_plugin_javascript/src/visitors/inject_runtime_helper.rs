use rspack_core::RuntimeGlobals;
use swc_core::common::{Mark, DUMMY_SP};
use swc_core::ecma::ast::*;
use swc_core::ecma::transforms::base::helpers::HELPERS;
use swc_core::ecma::utils::ExprFactory;
use swc_core::ecma::visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith};

use crate::utils::is_dynamic_import_literal_expr;

pub fn inject_runtime_helper(
  _unresolved_mark: Mark,
  runtime_requirements: &mut RuntimeGlobals,
) -> impl Fold + '_ {
  let helper_mark = HELPERS.with(|helper| helper.mark());
  as_folder(InjectRuntimeHelper {
    helper_mark,
    runtime_requirements,
  })
}

struct InjectRuntimeHelper<'a> {
  helper_mark: Mark,
  runtime_requirements: &'a mut RuntimeGlobals,
}

impl<'a> VisitMut for InjectRuntimeHelper<'a> {
  noop_visit_mut_type!();

  fn visit_mut_call_expr(&mut self, n: &mut CallExpr) {
    n.visit_mut_children_with(self);
    // dynamic import is used `interopRequire` runtime
    if is_dynamic_import_literal_expr(n) {
      self
        .runtime_requirements
        .insert(RuntimeGlobals::INTEROP_REQUIRE);
    }
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
          .insert(RuntimeGlobals::INTEROP_REQUIRE);
        n.callee = MemberExpr {
          span: DUMMY_SP,
          obj: Box::new(Expr::Ident(Ident::new(
            RuntimeGlobals::REQUIRE.into(),
            DUMMY_SP,
          ))),
          prop: MemberProp::Ident(Ident::new(RuntimeGlobals::INTEROP_REQUIRE.into(), DUMMY_SP)),
        }
        .as_callee();
        n.args.visit_mut_children_with(self);
        return;
      }

      if matches!(word, "_export_star") {
        self
          .runtime_requirements
          .insert(RuntimeGlobals::EXPORT_STAR);
        // TODO try with ast.parse(r#"self["__rspack_runtime__"].exportStar"#)
        n.callee = MemberExpr {
          span: DUMMY_SP,
          obj: Box::new(Expr::Ident(Ident::new(
            RuntimeGlobals::REQUIRE.into(),
            DUMMY_SP,
          ))),
          prop: MemberProp::Ident(Ident::new(RuntimeGlobals::EXPORT_STAR.into(), DUMMY_SP)),
        }
        .as_callee();
        n.args.visit_mut_children_with(self);
        return;
      }

      // the reason of adding this block you could refer to https://github.com/web-infra-dev/rspack/pull/2871#discussion_r1174505820
      if word == "_instanceof" {
        return;
      };
      // have some unhandled helper
      debug_assert!(false, "have unhandled helper: word = {word}");
    }
  }
}
