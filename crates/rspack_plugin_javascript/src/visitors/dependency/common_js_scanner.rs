use rspack_core::{
  DependencyLocation, DependencyTemplate, RuntimeGlobals, RuntimeRequirementsDependency,
};
use rustc_hash::FxHashSet;
use swc_core::common::SyntaxContext;
use swc_core::ecma::ast::{Expr, Ident};
use swc_core::ecma::visit::{noop_visit_type, Visit, VisitWith};

use super::expr_matcher;
use crate::no_visit_ignored_stmt;

pub struct CommonJsScanner<'a> {
  unresolved_ctxt: SyntaxContext,
  presentational_dependencies: &'a mut Vec<Box<dyn DependencyTemplate>>,
  has_module_ident: bool,
  ignored: &'a mut FxHashSet<DependencyLocation>,
}

impl<'a> CommonJsScanner<'a> {
  pub fn new(
    presentational_dependencies: &'a mut Vec<Box<dyn DependencyTemplate>>,
    unresolved_ctxt: SyntaxContext,
    ignored: &'a mut FxHashSet<DependencyLocation>,
  ) -> Self {
    Self {
      presentational_dependencies,
      unresolved_ctxt,
      has_module_ident: false,
      ignored,
    }
  }
}

impl Visit for CommonJsScanner<'_> {
  noop_visit_type!();
  no_visit_ignored_stmt!();

  fn visit_ident(&mut self, ident: &Ident) {
    if self.has_module_ident {
      return;
    }
    if &ident.sym == "module" && ident.span.ctxt == self.unresolved_ctxt {
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
