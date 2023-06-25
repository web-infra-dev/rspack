use rspack_core::{CodeGeneratableDependency, RuntimeGlobals, RuntimeRequirementsDependency};
use swc_core::common::pass::AstNodePath;
use swc_core::ecma::ast::{CallExpr, Expr, Lit};
use swc_core::ecma::visit::fields::IfStmtField;
use swc_core::ecma::visit::{AstParentKind, AstParentNodeRef, VisitAstPath, VisitWithPath};
use swc_core::ecma::{
  ast::Expr,
  visit::{noop_visit_type, Visit, VisitWith},
};
use swc_core::quote;

use super::expr_matcher;

pub struct CommonJsScanner<'a> {
  presentational_dependencies: &'a mut Vec<Box<dyn CodeGeneratableDependency>>,
}

impl<'a> CommonJsScanner<'a> {
  pub fn new(presentational_dependencies: &'a mut Vec<Box<dyn CodeGeneratableDependency>>) -> Self {
    Self {
      presentational_dependencies,
    }
  }
}

impl Visit for CommonJsScanner<'_> {
  noop_visit_type!();

  fn visit_expr(&mut self, expr: &Expr) {
    if expr_matcher::is_module_id(expr) {
      self
        .presentational_dependencies
        .push(Box::new(RuntimeRequirementsDependency::new(
          RuntimeGlobals::MODULE_ID,
        )));
    }
    if expr_matcher::is_module_loaded(expr) {
      self
        .presentational_dependencies
        .push(Box::new(RuntimeRequirementsDependency::new(
          RuntimeGlobals::MODULE_LOADED,
        )));
    }
    expr.visit_children_with(self);
  }
}
