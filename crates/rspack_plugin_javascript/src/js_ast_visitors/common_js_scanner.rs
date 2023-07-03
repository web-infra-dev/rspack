use rspack_core::{CodeGeneratableDependency, RuntimeGlobals, RuntimeRequirementsDependency};
use rspack_plugin_javascript_shared::JsAstVisitorHook;
use swc_core::ecma::ast::Expr;

use crate::visitors::expr_matcher;

#[derive(Default)]
pub struct CommonJsScanner {
  pub(crate) presentational_dependencies: Vec<Box<dyn CodeGeneratableDependency>>,
}

impl JsAstVisitorHook for CommonJsScanner {
  fn visit_expr(&mut self, expr: &Expr) -> bool {
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
    false
  }
}
