use swc_core::ecma::ast::{ClassExpr, Ident, Prop};
use swc_core::ecma::visit::{noop_visit_type, Visit, VisitWith};

#[derive(Clone, Debug)]
pub struct ConcatenatedModuleIdent {
  pub id: Ident,
  pub shorthand: bool,
  pub class_expr_with_ident: bool,
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
      class_expr_with_ident: false,
    });
  }

  fn visit_prop(&mut self, node: &Prop) {
    match node {
      Prop::Shorthand(node) => {
        self.ids.push(ConcatenatedModuleIdent {
          id: node.clone(),
          shorthand: true,
          class_expr_with_ident: false,
        });
      }
      _ => {
        node.visit_children_with(self);
      }
    }
  }

  fn visit_class_expr(&mut self, node: &ClassExpr) {
    if let Some(ref ident) = node.ident
      && node.class.super_class.is_some()
    {
      self.ids.push(ConcatenatedModuleIdent {
        id: ident.clone(),
        shorthand: false,
        class_expr_with_ident: true,
      });
    }
    node.class.visit_with(self);
  }
}
