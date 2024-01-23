use swc_core::ecma::ast::{Ident, Prop};
use swc_core::ecma::visit::{noop_visit_type, Visit, VisitWith};

#[derive(Clone, Debug)]
pub struct ConcatenatedModuleIdent {
  pub id: Ident,
  pub shorthand: bool,
}

#[derive(Default)]
pub struct IdentCollector {
  pub ids: Vec<ConcatenatedModuleIdent>,
}

impl IdentCollector {
  pub fn new(ids: Vec<ConcatenatedModuleIdent>) -> Self {
    Self { ids }
  }
}

impl Visit for IdentCollector {
  noop_visit_type!();

  fn visit_ident(&mut self, node: &Ident) {
    self.ids.push(ConcatenatedModuleIdent {
      id: node.clone(),
      shorthand: false,
    });
  }

  fn visit_prop(&mut self, node: &Prop) {
    match node {
      Prop::Shorthand(node) => {
        self.ids.push(ConcatenatedModuleIdent {
          id: node.clone(),
          shorthand: true,
        });
      }
      _ => {
        node.visit_children_with(self);
      }
    }
  }
}
