use rspack_core::{ModuleDependency, Provide, SpanExt};
use swc_core::{
  common::SyntaxContext,
  ecma::{
    ast::{Expr, Ident, MemberExpr},
    atoms::JsWord,
    visit::{noop_visit_type, Visit, VisitWith},
  },
};

use crate::dependency::ProvideDependency;

pub struct ProvideScanner<'a> {
  opts: &'a Provide,
  unresolved_ctxt: &'a SyntaxContext,
  dependencies: &'a mut Vec<Box<dyn ModuleDependency>>,
}

impl<'a> ProvideScanner<'a> {
  pub fn new(
    opts: &'a Provide,
    unresolved_ctxt: &'a SyntaxContext,
    dependencies: &'a mut Vec<Box<dyn ModuleDependency>>,
  ) -> Self {
    Self {
      opts,
      unresolved_ctxt,
      dependencies,
    }
  }

  fn get_nested_identifier_name(&self, member_expr: &MemberExpr) -> Option<String> {
    let mut obj: String = match &*member_expr.obj {
      Expr::Member(nested_member_expr) => self.get_nested_identifier_name(nested_member_expr),
      Expr::Ident(ident) => {
        if ident.span.ctxt == *self.unresolved_ctxt {
          Some(ident.sym.to_string())
        } else {
          None
        }
      }
      Expr::This(_) => Some("this".to_string()),
      _ => None,
    }?;

    if let Some(ident_prop) = member_expr.prop.as_ident() {
      obj.push('.');
      obj.push_str(&ident_prop.sym);
      return Some(obj);
    }
    None
  }

  fn get_resolved_request(&self, value: &str) -> Option<(JsWord, Vec<String>)> {
    self.opts.get(value).map(|requests| {
      (
        requests.get(0).expect("should have request").clone().into(),
        requests.iter().skip(1).cloned().collect::<Vec<String>>(),
      )
    })
  }
}

impl Visit for ProvideScanner<'_> {
  noop_visit_type!();

  fn visit_ident(&mut self, ident: &Ident) {
    if let Some((request, ids)) = self.get_resolved_request(ident.sym.as_ref()) && ident.span.ctxt == *self.unresolved_ctxt {
      self.dependencies.push(Box::new(ProvideDependency::new(
        ident.span.real_lo(),
        ident.span.real_hi(),
        ids,
        request,
        ident.sym.clone()
      )));
    }
  }

  fn visit_member_expr(&mut self, member_expr: &MemberExpr) {
    if let Some(identifier_name) = self.get_nested_identifier_name(member_expr) && let Some((request, ids)) = self.get_resolved_request(&identifier_name) {
        self.dependencies.push(Box::new(ProvideDependency::new(
            member_expr.span.real_lo(),
            member_expr.span.real_hi(),
            ids,
            request,
            identifier_name.replace(".", "_dot_").into()
          )));
    } else {
        member_expr.visit_children_with(self);
    }
  }
}
