use rspack_core::{
  extract_member_expression_chain, ConstDependency, DependencyLocation, DependencyTemplate, SpanExt,
};
use rustc_hash::FxHashSet;
use swc_core::{
  common::SyntaxContext,
  ecma::{
    ast::*,
    visit::{noop_visit_type, Visit},
  },
};

use crate::{dependency::ExportInfoApiDependency, no_visit_ignored_stmt};

pub struct ExportInfoApiScanner<'a> {
  pub presentational_dependencies: &'a mut Vec<Box<dyn DependencyTemplate>>,
  unresolved_ctxt: SyntaxContext,
  pub ignored: &'a mut FxHashSet<DependencyLocation>,
}

//__webpack_exports_info__.a.used
impl<'a> ExportInfoApiScanner<'a> {
  pub fn new(
    presentational_dependencies: &'a mut Vec<Box<dyn DependencyTemplate>>,
    unresolved_ctxt: SyntaxContext,
    ignored: &'a mut FxHashSet<DependencyLocation>,
  ) -> Self {
    Self {
      presentational_dependencies,
      unresolved_ctxt,
      ignored,
    }
  }
}

impl Visit for ExportInfoApiScanner<'_> {
  noop_visit_type!();
  no_visit_ignored_stmt!();

  fn visit_member_expr(&mut self, member_expr: &MemberExpr) {
    let expression_info = extract_member_expression_chain(member_expr);
    let member_chain = expression_info.members();
    if !member_chain.is_empty()
      && &member_chain[0].0 == "__webpack_exports_info__"
      && member_chain[0].1 == self.unresolved_ctxt
    {
      let len = member_chain.len();
      if len >= 3 {
        let prop = member_chain[len - 1].0.clone();
        let dep = Box::new(ExportInfoApiDependency::new(
          member_expr.span.real_lo(),
          member_expr.span.real_hi(),
          member_chain
            .into_iter()
            .skip(1)
            .take(len - 2)
            .map(|item| item.0.clone())
            .collect::<Vec<_>>(),
          prop,
        ));
        self.presentational_dependencies.push(dep);
      } else {
        // TODO: support other __webpack_exports_info__
      }
    }
  }

  fn visit_ident(&mut self, n: &Ident) {
    if n.sym == "__webpack_exports_info__" {
      let dep = Box::new(ConstDependency::new(
        n.span.real_lo(),
        n.span.real_hi(),
        "true".into(),
        None,
      ));
      self.presentational_dependencies.push(dep);
    }
  }
}
