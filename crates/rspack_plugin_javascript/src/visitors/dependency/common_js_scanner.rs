use rspack_core::{Dependency, RuntimeGlobals, RuntimeRequirementsDependency};
use swc_core::ecma::ast::Expr;
use swc_core::ecma::visit::{noop_visit_type, Visit, VisitWith};

use super::match_member_expr;

pub struct CommonJsScanner<'a> {
  pub presentational_dependencies: &'a mut Vec<Box<dyn Dependency>>,
}

impl<'a> CommonJsScanner<'a> {
  pub fn new(presentational_dependencies: &'a mut Vec<Box<dyn Dependency>>) -> Self {
    Self {
      presentational_dependencies,
    }
  }

  fn add_presentational_dependency(&mut self, dependency: Box<dyn Dependency>) {
    self.presentational_dependencies.push(dependency);
  }
}

impl Visit for CommonJsScanner<'_> {
  noop_visit_type!();

  fn visit_expr(&mut self, expr: &Expr) {
    if match_member_expr(&expr, "module.id") {
      self.add_presentational_dependency(box RuntimeRequirementsDependency::new(
        RuntimeGlobals::MODULE_ID,
      ));
    }
    if match_member_expr(&expr, "module.loaded") {
      self.add_presentational_dependency(box RuntimeRequirementsDependency::new(
        RuntimeGlobals::MODULE_LOADED,
      ));
    }
    expr.visit_children_with(self);
  }
}
