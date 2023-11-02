use rspack_core::{ConstDependency, DependencyTemplate, SpanExt};
use swc_core::ecma::{
  ast::{
    ClassMember, ClassMethod, ClassProp, Expr, Function, GetterProp, MethodProp, Prop, PropName,
    SetterProp, ThisExpr,
  },
  visit::{noop_visit_type, Visit, VisitWith},
};

pub struct HarmonyTopLevelThis<'a> {
  pub presentational_dependencies: &'a mut Vec<Box<dyn DependencyTemplate>>,
}

impl Visit for HarmonyTopLevelThis<'_> {
  noop_visit_type!();

  fn visit_function(&mut self, _: &Function) {}

  fn visit_class_member(&mut self, n: &ClassMember) {
    match n {
      ClassMember::Method(ClassMethod {
        key: PropName::Computed(computed),
        ..
      })
      | ClassMember::ClassProp(ClassProp {
        key: PropName::Computed(computed),
        ..
      }) => {
        computed.visit_with(self);
      }
      _ => {}
    }
  }

  fn visit_prop(&mut self, n: &Prop) {
    match n {
      Prop::KeyValue(..) => {
        n.visit_children_with(self);
      }
      Prop::Getter(GetterProp {
        key: PropName::Computed(computed),
        ..
      })
      | Prop::Setter(SetterProp {
        key: PropName::Computed(computed),
        ..
      })
      | Prop::Method(MethodProp {
        key: PropName::Computed(computed),
        ..
      }) => computed.visit_children_with(self),
      _ => {}
    }
  }

  fn visit_expr(&mut self, n: &Expr) {
    if let Expr::This(ThisExpr { span }) = n {
      self
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          span.real_lo(),
          span.real_hi(),
          "undefined".into(),
          None,
        )));
    } else {
      n.visit_children_with(self);
    }
  }
}
