use swc_common::SyntaxContext;
use swc_ecma_ast::Ident;
use swc_ecma_visit::{as_folder, noop_visit_mut_type, Fold, VisitMut};

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
