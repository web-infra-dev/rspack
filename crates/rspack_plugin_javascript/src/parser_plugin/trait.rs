use swc_core::ecma::ast::{CallExpr, Ident, UnaryExpr};

use crate::utils::eval::BasicEvaluatedExpression;
use crate::visitors::common_js_import_dependency_scanner::CommonJsImportDependencyScanner;

pub trait JavascriptParserPlugin {
  fn evaluate_typeof(
    &self,
    _ident: &Ident,
    _start: u32,
    _end: u32,
    _unresolved_mark: swc_core::common::SyntaxContext, // remove this after `parser.scope.definitions`
  ) -> Option<BasicEvaluatedExpression> {
    None
  }

  fn call(
    &self,
    _parser: &mut CommonJsImportDependencyScanner<'_>,
    _expr: &CallExpr,
  ) -> Option<bool> {
    None
  }

  fn r#typeof(
    &self,
    _parser: &mut CommonJsImportDependencyScanner<'_>,
    _expr: &UnaryExpr,
  ) -> Option<bool> {
    None
  }
}

pub type BoxJavascriptParserPlugin = Box<dyn JavascriptParserPlugin>;
