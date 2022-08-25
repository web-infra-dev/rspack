use rspack_core::Compilation;
use swc_atoms::JsWord;
use swc_common::{EqIgnoreSpan, DUMMY_SP};
use swc_ecma_ast::{CallExpr, Expr, Str};
use swc_ecma_utils::member_expr;
use swc_ecma_visit::{noop_visit_mut_type, VisitMut, VisitMutWith};

pub struct HmrModuleIdReWriter<'a> {
  pub rewriting: bool,
  pub compilation: &'a Compilation,
}

impl<'a> VisitMut for HmrModuleIdReWriter<'a> {
  noop_visit_mut_type!();
  fn visit_mut_call_expr(&mut self, call_expr: &mut CallExpr) {
    call_expr
      .callee
      .as_mut_expr()
      .and_then(|expr| match &mut **expr {
        Expr::Member(member) => Some(member),
        _ => None,
      })
      .and_then(|member| match *member_expr!(DUMMY_SP, module.hot.accpet) {
        Expr::Member(expr) => expr.eq_ignore_span(member).then_some(()),
        _ => None,
      })
      .and_then(|_| {
        println!("before modify rewrite {:?}", call_expr);
        self.rewriting = true;
        let call_expr_len = call_expr.args.len();
        // exclude last elements of `module.hot.accpet`
        for expr_or_spread in call_expr.args.iter_mut().take(call_expr_len - 1).rev() {
          expr_or_spread.visit_mut_with(self);
        }
        call_expr.visit_mut_children_with(self);
        self.rewriting = false;
        Some(())
      })
      .unwrap_or_else(|| {
        call_expr.visit_mut_children_with(self);
      });
  }

  fn visit_mut_str(&mut self, str: &mut Str) {
    if self.rewriting {
      println!("need rewrite {:?}", &str.value);
      // if let Some(rid) = self.resolved_ids.get(&str.value) {
      //   let uri = &rid.uri;
      //   let js_module = self.compilation.module_graph.module_by_uri(uri).unwrap();
      //   let id = js_module.id.as_str();
      //   str.value = JsWord::from(id);
      //   str.raw = Some(JsWord::from(format!("\"{}\"", id)));
      // }
    }
  }
}
