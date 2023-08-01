use std::collections::VecDeque;

use rspack_core::{ExportInfo, ModuleDependency, SpanExt};
use swc_core::{
  common::{Mark, SyntaxContext},
  ecma::{
    ast::*,
    atoms::JsWord,
    visit::{noop_visit_type, Visit, VisitWith},
  },
};

use crate::dependency::{ExportInfoApiDependency, URLDependency};

pub struct ExportInfoApiScanner<'a> {
  pub dependencies: &'a mut Vec<Box<dyn ModuleDependency>>,
  unresolved_ctxt: SyntaxContext,
}

//__webpack_exports_info__.a.used
impl<'a> ExportInfoApiScanner<'a> {
  pub fn new(
    dependencies: &'a mut Vec<Box<dyn ModuleDependency>>,
    unresolved_ctxt: SyntaxContext,
  ) -> Self {
    Self {
      dependencies,
      unresolved_ctxt,
    }
  }
}

impl Visit for ExportInfoApiScanner<'_> {
  noop_visit_type!();

  fn visit_member_expr(&mut self, member_expr: &MemberExpr) {
    let member_chain = extract_member_expression_chain(member_expr);

    if &member_chain[0].0 == "__webpack_exports_info__" && member_chain[0].1 == self.unresolved_ctxt
    {
      let len = member_chain.len();
      if len >= 3 {
        dbg!(&member_chain);
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
        self.dependencies.push(dep);
      } else {
        // TODO: support other __webpack_exports_info__
      }
    }
  }
}

fn extract_member_expression_chain(expression: &MemberExpr) -> VecDeque<(JsWord, SyntaxContext)> {
  let mut members: VecDeque<(JsWord, SyntaxContext)> = VecDeque::new();
  let mut expr = expression;

  loop {
    if let MemberProp::Computed(ComputedPropName {
      expr: box Expr::Lit(Lit::Str(ref val)),
      ..
    }) = expr.prop
    {
      members.push_front((val.value.clone(), val.span.ctxt));
    } else if let MemberProp::Ident(ref ident) = expr.prop {
      members.push_front((ident.sym.clone(), ident.span.ctxt));
    } else {
      break;
    }
    match expr.obj {
      box Expr::Member(ref member_expr) => {
        expr = member_expr;
      }
      box Expr::Ident(ref ident) => {
        members.push_front((ident.sym.clone(), ident.span.ctxt));
        break;
      }
      _ => break,
    }
  }
  members
}
