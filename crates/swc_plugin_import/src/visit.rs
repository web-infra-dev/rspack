use rustc_hash::FxHashSet as HashSet;
use swc_core::ecma::{
  ast::{Id, Ident, ImportDecl, TsTypeRef},
  visit::{Visit, VisitWith},
};

#[derive(Default)]
pub(crate) struct IdentComponent {
  pub(crate) ident_set: HashSet<Id>,
  pub(crate) type_ident_set: HashSet<Id>,
  pub(crate) in_ts_type_ref: bool,
}

///
/// track ident reference
impl Visit for IdentComponent {
  // need to skip import decl
  fn visit_import_decl(&mut self, _: &ImportDecl) {}

  fn visit_ident(&mut self, ident: &Ident) {
    if self.in_ts_type_ref {
      self.type_ident_set.insert(ident.to_id());
    } else {
      self.ident_set.insert(ident.to_id());
    }
  }

  fn visit_ts_type_ref(&mut self, type_ref: &TsTypeRef) {
    let store_in_ts_type_ref = self.in_ts_type_ref;
    self.in_ts_type_ref = true;

    type_ref.type_name.visit_with(self);

    self.in_ts_type_ref = store_in_ts_type_ref;
  }
}
