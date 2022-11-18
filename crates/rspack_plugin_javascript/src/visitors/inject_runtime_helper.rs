use hashbrown::HashSet;
use rspack_core::runtime_globals;
use swc_common::{Mark, DUMMY_SP};
use swc_ecma_ast::*;
use swc_ecma_transforms::helpers::HELPERS;
use swc_ecma_utils::ExprFactory;
use swc_ecma_visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith};

pub fn inject_runtime_helper(runtime_requirements: &mut HashSet<String>) -> impl Fold + '_ {
  let helper_mark = HELPERS.with(|helper| helper.mark());
  as_folder(InjectRuntimeHelper {
    helper_mark,
    runtime_requirements,
  })
}

struct InjectRuntimeHelper<'a> {
  helper_mark: Mark,
  runtime_requirements: &'a mut HashSet<String>,
}

impl<'a> VisitMut for InjectRuntimeHelper<'a> {
  noop_visit_mut_type!();

  fn visit_mut_call_expr(&mut self, n: &mut CallExpr) {
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
