use swc_core::{
  common::SyntaxContext,
  ecma::{
    ast::Ident,
    visit::{as_folder, noop_visit_mut_type, Fold, VisitMut},
  },
};

pub fn clear_mark() -> impl Fold {
  as_folder(ClearMark {})
}

struct ClearMark;
impl VisitMut for ClearMark {
  noop_visit_mut_type!();

  fn visit_mut_ident(&mut self, ident: &mut Ident) {
    ident.span.ctxt = SyntaxContext::empty();
  }
}
