use rspack_core::{
  ConstDependency, DependencyLocation, DependencyTemplate, RuntimeGlobals,
  RuntimeRequirementsDependency, SpanExt,
};
use swc_core::common::{Spanned, SyntaxContext};
use swc_core::ecma::ast::{Expr, Ident};
use swc_core::ecma::visit::{noop_visit_type, Visit, VisitWith};

use super::expr_matcher;
use crate::no_visit_ignored_stmt;

pub struct CommonJsScanner<'a> {
  unresolved_ctxt: SyntaxContext,
  presentational_dependencies: &'a mut Vec<Box<dyn DependencyTemplate>>,
  has_module_ident: bool,
  ignored: &'a mut Vec<DependencyLocation>,
}

impl<'a> CommonJsScanner<'a> {
  pub fn new(
    presentational_dependencies: &'a mut Vec<Box<dyn DependencyTemplate>>,
    unresolved_ctxt: SyntaxContext,
    ignored: &'a mut Vec<DependencyLocation>,
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
    if expr_matcher::is_require_main(expr) {
      let mut runtime_requirements = RuntimeGlobals::default();
      runtime_requirements.insert(RuntimeGlobals::MODULE_CACHE);
      runtime_requirements.insert(RuntimeGlobals::ENTRY_MODULE_ID);
      self
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          expr.span().real_lo(),
          expr.span().real_hi(),
          format!(
            "{}[{}]",
            RuntimeGlobals::MODULE_CACHE,
            RuntimeGlobals::ENTRY_MODULE_ID
          )
          .into(),
          Some(runtime_requirements),
        )));
    }
    expr.visit_children_with(self);
  }
}
