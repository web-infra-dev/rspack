use rspack_core::runtime_globals;
use rustc_hash::FxHashSet as HashSet;
use swc_core::common::{Mark, DUMMY_SP};
use swc_core::ecma::ast::*;
use swc_core::ecma::transforms::base::helpers::HELPERS;
use swc_core::ecma::utils::ExprFactory;
use swc_core::ecma::visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith};

use crate::utils::is_dynamic_import_literal_expr;

pub fn inject_runtime_helper<'a>(
  unresolved_mark: Mark,
  runtime_requirements: &'a mut HashSet<&'static str>,
) -> impl Fold + 'a {
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
  runtime_requirements: &'a mut HashSet<&'static str>,
}

impl<'a> VisitMut for InjectRuntimeHelper<'a> {
  noop_visit_mut_type!();

  fn visit_mut_call_expr(&mut self, n: &mut CallExpr) {
    n.visit_mut_children_with(self);
    // dynamic import is used `interopRequire` runtime
    if is_dynamic_import_literal_expr(n) {
      self
        .runtime_requirements
        .insert(runtime_globals::INTEROP_REQUIRE);
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
          .insert(runtime_globals::INTEROP_REQUIRE);
        n.callee = MemberExpr {
          span: DUMMY_SP,
          obj: Box::new(Expr::Ident(Ident::new(
            runtime_globals::REQUIRE.into(),
            DUMMY_SP,
          ))),
          prop: MemberProp::Ident(Ident::new(
            runtime_globals::INTEROP_REQUIRE.into(),
            DUMMY_SP,
          )),
        }
        .as_callee();
        n.args.visit_mut_children_with(self);
        return;
      }

      if matches!(word, "_export_star") {
        self
          .runtime_requirements
          .insert(runtime_globals::EXPORT_STAR);
        // TODO try with ast.parse(r#"self["__rspack_runtime__"].exportStar"#)
        n.callee = MemberExpr {
          span: DUMMY_SP,
          obj: Box::new(Expr::Ident(Ident::new(
            runtime_globals::REQUIRE.into(),
            DUMMY_SP,
          ))),
          prop: MemberProp::Ident(Ident::new(runtime_globals::EXPORT_STAR.into(), DUMMY_SP)),
        }
        .as_callee();
        n.args.visit_mut_children_with(self);
        return;
      }

      // have some unhandled helper
      debug_assert!(false, "have unhandled helper: word = {word}");
    }
  }
}
