use std::collections::HashMap;

use crate::{Bundle, ResolvedURI};
use ast::*;
use rspack_swc::{swc_atoms, swc_common, swc_ecma_ast as ast, swc_ecma_utils, swc_ecma_visit};
use swc_atoms::JsWord;
use swc_common::{EqIgnoreSpan, DUMMY_SP};
use swc_ecma_utils::member_expr;
use swc_ecma_visit::{VisitMut, VisitMutWith};

pub struct HmrModuleIdReWriter<'a> {
  pub resolved_ids: &'a HashMap<JsWord, ResolvedURI>,
  pub rewriting: bool,
  pub bundle: &'a Bundle,
}

impl<'a> VisitMut for HmrModuleIdReWriter<'a> {
  fn visit_mut_call_expr(&mut self, call_expr: &mut CallExpr) {
    if let Callee::Expr(expr) = &mut call_expr.callee {
      match &mut **expr {
        Expr::Member(member_expr) => {
          if let Expr::Member(expr) = *member_expr!(DUMMY_SP, module.hot.accpet) {
            if expr.eq_ignore_span(member_expr) {
              self.rewriting = true;

              let call_expr_len = call_expr.args.len();
              // exclude last elements of `module.hot.accpet`
              for expr_or_spread in call_expr.args.iter_mut().take(call_expr_len - 1).rev() {
                expr_or_spread.visit_mut_with(self);
              }

              call_expr.visit_mut_children_with(self);
              self.rewriting = false;
            } else {
              call_expr.visit_mut_children_with(self);
            }
          } else {
            call_expr.visit_mut_children_with(self);
          }
        }
        _ => call_expr.visit_mut_children_with(self),
      }
    } else {
      call_expr.visit_mut_children_with(self)
    }
  }
  fn visit_mut_str(&mut self, str: &mut Str) {
    if self.rewriting {
      if let Some(rid) = self.resolved_ids.get(&str.value) {
        let uri = &rid.uri;
        let js_module = self
          .bundle
          .module_graph_container
          .module_graph
          .module_by_uri(uri)
          .unwrap();
        str.value = JsWord::from(js_module.id.as_str());
        str.raw = Some(JsWord::from(format!("\"{}\"", js_module.id)));
      }
    }
  }
}
