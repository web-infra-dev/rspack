use swc_core::ecma::ast::{ClassExpr, Ident, ObjectPatProp, Prop};
use swc_core::ecma::visit::{noop_visit_type, Visit, VisitWith};

#[derive(Clone, Debug)]
pub struct ConcatenatedModuleIdent {
  pub id: Ident,
  pub shorthand: bool,
  pub is_class_expr_with_ident: bool,
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
      is_class_expr_with_ident: false,
    });
  }

  fn visit_object_pat_prop(&mut self, n: &ObjectPatProp) {
    match n {
      ObjectPatProp::Assign(assign) => {
        self.ids.push(ConcatenatedModuleIdent {
          id: assign.key.clone().into(),
          shorthand: true,
          is_class_expr_with_ident: false,
        });
        assign.value.visit_with(self);
      }
      ObjectPatProp::KeyValue(_) | ObjectPatProp::Rest(_) => {
        n.visit_children_with(self);
      }
    }
  }

  fn visit_prop(&mut self, node: &Prop) {
    match node {
      Prop::Shorthand(node) => {
        self.ids.push(ConcatenatedModuleIdent {
          id: node.clone(),
          shorthand: true,
          is_class_expr_with_ident: false,
        });
      }
      _ => {
        node.visit_children_with(self);
      }
    }
  }

  /// https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/optimize/ConcatenatedModule.js#L1173-L1197
  fn visit_class_expr(&mut self, node: &ClassExpr) {
    if let Some(ref ident) = node.ident
      && node.class.super_class.is_some()
    {
      self.ids.push(ConcatenatedModuleIdent {
        id: ident.clone(),
        shorthand: false,
        is_class_expr_with_ident: true,
      });
    }
    node.class.visit_with(self);
  }
}
