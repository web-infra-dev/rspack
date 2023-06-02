use rspack_core::{CodeReplaceSourceDependency, RuntimeGlobals, RuntimeRequirementsDependency};
use swc_core::ecma::{
  ast::Expr,
  visit::{noop_visit_type, Visit, VisitWith},
};

use super::expr_matcher;

pub struct NewCommonJsScanner<'a> {
  code_generable_dependencies: &'a mut Vec<Box<dyn CodeReplaceSourceDependency>>,
}

impl<'a> NewCommonJsScanner<'a> {
  pub fn new(
    code_generable_dependencies: &'a mut Vec<Box<dyn CodeReplaceSourceDependency>>,
  ) -> Self {
    Self {
      code_generable_dependencies,
    }
  }
}

impl Visit for NewCommonJsScanner<'_> {
  noop_visit_type!();

  fn visit_expr(&mut self, expr: &Expr) {
    if expr_matcher::is_module_id(expr) {
      self
        .code_generable_dependencies
        .push(Box::new(RuntimeRequirementsDependency::new(
          RuntimeGlobals::MODULE_ID,
        )));
    }
    if expr_matcher::is_module_loaded(expr) {
      self
        .code_generable_dependencies
        .push(Box::new(RuntimeRequirementsDependency::new(
          RuntimeGlobals::MODULE_LOADED,
        )));
    }
    expr.visit_children_with(self);
  }
}
