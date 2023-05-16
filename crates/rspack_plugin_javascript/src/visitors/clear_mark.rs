use swc_core::{
  common::SyntaxContext,
  ecma::{
    ast::Ident,
    visit::{noop_visit_mut_type, VisitMut},
  },
};

struct ClearMark;

impl VisitMut for ClearMark {
  noop_visit_mut_type!();

  fn visit_mut_ident(&mut self, ident: &mut Ident) {
    ident.span.ctxt = SyntaxContext::empty();
  }
}
