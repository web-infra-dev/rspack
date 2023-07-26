use rspack_core::{DependencyTemplate, RuntimeGlobals, RuntimeRequirementsDependency};
use swc_core::common::SyntaxContext;
use swc_core::ecma::ast::{Expr, Ident};
use swc_core::ecma::visit::{noop_visit_type, Visit, VisitWith};

use super::expr_matcher;

pub struct CommonJsScanner<'a> {
  unresolved_ctxt: &'a SyntaxContext,
  presentational_dependencies: &'a mut Vec<Box<dyn DependencyTemplate>>,
  has_module_ident: bool,
}

impl<'a> CommonJsScanner<'a> {
  pub fn new(
    presentational_dependencies: &'a mut Vec<Box<dyn DependencyTemplate>>,
    unresolved_ctxt: &'a SyntaxContext,
  ) -> Self {
    Self {
      presentational_dependencies,
      unresolved_ctxt,
      has_module_ident: false,
    }
  }
}

impl Visit for CommonJsScanner<'_> {
  noop_visit_type!();

  fn visit_ident(&mut self, ident: &Ident) {
    if self.has_module_ident {
      return;
    }
    if &ident.sym == "module" && ident.span.ctxt == *self.unresolved_ctxt {
      self
        .presentational_dependencies
        .push(Box::new(RuntimeRequirementsDependency::new(
          RuntimeGlobals::MODULE,
        )));
      self.has_module_ident = true;
    }
  }

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
