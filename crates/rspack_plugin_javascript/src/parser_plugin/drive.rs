use swc_core::ecma::ast::CallExpr;

use super::{BoxJavascriptParserPlugin, JavascriptParserPlugin};
use crate::utils::eval::BasicEvaluatedExpression;
use crate::visitors::common_js_import_dependency_scanner::CommonJsImportDependencyScanner;

pub struct JavaScriptParserPluginDrive {
  plugins: Vec<BoxJavascriptParserPlugin>,
}

impl JavaScriptParserPluginDrive {
  pub fn new(plugins: Vec<BoxJavascriptParserPlugin>) -> Self {
    Self { plugins }
  }
}

impl JavascriptParserPlugin for JavaScriptParserPluginDrive {
  fn evaluate_typeof(
    &self,
    ident: &swc_core::ecma::ast::Ident,
    start: u32,
    end: u32,
    unresolved_mark: swc_core::common::SyntaxContext,
  ) -> Option<BasicEvaluatedExpression> {
    for plugin in &self.plugins {
      let res = plugin.evaluate_typeof(ident, start, end, unresolved_mark);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn call(&self, parser: &mut CommonJsImportDependencyScanner, expr: &CallExpr) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.call(parser, expr);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn r#typeof(
    &self,
    parser: &mut CommonJsImportDependencyScanner<'_>,
    expr: &swc_core::ecma::ast::UnaryExpr,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.r#typeof(parser, expr);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }
}
