use rspack_core::{extract_member_expression_chain, DependencyTemplate, SpanExt};
use swc_core::{
  common::SyntaxContext,
  ecma::{
    ast::*,
    visit::{noop_visit_type, Visit},
  },
};

use crate::dependency::ExportInfoApiDependency;

pub struct ExportInfoApiScanner<'a> {
  pub presentational_dependencies: &'a mut Vec<Box<dyn DependencyTemplate>>,
  unresolved_ctxt: SyntaxContext,
}

//__webpack_exports_info__.a.used
impl<'a> ExportInfoApiScanner<'a> {
  pub fn new(
    presentational_dependencies: &'a mut Vec<Box<dyn DependencyTemplate>>,
    unresolved_ctxt: SyntaxContext,
  ) -> Self {
    Self {
      presentational_dependencies,
      unresolved_ctxt,
    }
  }
}

impl Visit for ExportInfoApiScanner<'_> {
  noop_visit_type!();

  fn visit_member_expr(&mut self, member_expr: &MemberExpr) {
    let member_chain = extract_member_expression_chain(member_expr);
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
            .map(|item| (item.0))
            .collect::<Vec<_>>(),
          prop,
        ));
        self.presentational_dependencies.push(dep);
      } else {
        // TODO: support other __webpack_exports_info__
      }
    }
  }
}
