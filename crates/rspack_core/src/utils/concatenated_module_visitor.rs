use swc_core::ecma::{
  ast::{ClassExpr, Ident, ObjectPatProp, Prop},
  visit::{Visit, VisitWith, noop_visit_type},
};

#[derive(Clone, Debug)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub struct ConcatenatedModuleIdent {
  pub id: Ident,
  pub shorthand: bool,
  pub is_class_expr_with_ident: bool,
}

#[derive(Default)]
pub struct IdentCollector {
  pub ids: Vec<ConcatenatedModuleIdent>,
}

impl IdentCollector {}

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

  /// Reserve every class expression's inner name, even without a superclass,
  /// so renaming an outer binding cannot make it captured by the class scope.
  fn visit_class_expr(&mut self, node: &ClassExpr) {
    if let Some(ref ident) = node.ident {
      self.ids.push(ConcatenatedModuleIdent {
        id: ident.clone(),
        shorthand: false,
        is_class_expr_with_ident: true,
      });
    }
    node.class.visit_with(self);
  }
}
