use swc_core::ecma::ast::{BinExpr, CallExpr, IfStmt, VarDecl, VarDeclarator};

use super::BoxJavascriptParserPlugin;
use crate::parser_plugin::r#const::is_logic_op;
use crate::utils::eval::BasicEvaluatedExpression;
use crate::visitors::JavascriptParser;

pub struct JavaScriptParserPluginDrive<'ast, 'parser> {
  plugins: Vec<BoxJavascriptParserPlugin<'ast, 'parser>>,
}

impl<'ast, 'parser> JavaScriptParserPluginDrive<'ast, 'parser> {
  pub fn new(plugins: Vec<BoxJavascriptParserPlugin<'ast, 'parser>>) -> Self {
    Self { plugins }
  }

  pub fn evaluate_typeof(
    &self,
    parser: &mut JavascriptParser<'parser>,
    ident: &'ast swc_core::ecma::ast::Ident,
    start: u32,
    end: u32,
  ) -> Option<BasicEvaluatedExpression<'ast>> {
    for plugin in &self.plugins {
      let res = plugin.evaluate_typeof(parser, ident, start, end, self);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  pub fn call(&self, parser: &mut JavascriptParser<'parser>, expr: &'ast CallExpr) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.call(parser, expr, self);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  pub fn member(
    &self,
    parser: &mut JavascriptParser<'parser>,
    expr: &'ast swc_core::ecma::ast::MemberExpr,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.member(parser, expr, self);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  pub fn r#typeof(
    &self,
    parser: &mut JavascriptParser<'parser>,
    expr: &'ast swc_core::ecma::ast::UnaryExpr,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.r#typeof(parser, expr, self);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  pub fn expression_logical_operator(
    &self,
    parser: &mut JavascriptParser<'parser>,
    expr: &'ast BinExpr,
  ) -> Option<bool> {
    assert!(is_logic_op(expr.op));
    for plugin in &self.plugins {
      let res = plugin.expression_logical_operator(parser, expr, self);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  pub fn binary_expression(
    &self,
    parser: &mut JavascriptParser<'parser>,
    expr: &'ast BinExpr,
  ) -> Option<bool> {
    assert!(!is_logic_op(expr.op));
    for plugin in &self.plugins {
      let res = plugin.binary_expression(parser, expr, self);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  pub fn statement_if(
    &self,
    parser: &mut JavascriptParser<'parser>,
    expr: &'ast IfStmt,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.statement_if(parser, expr, self);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  pub fn declarator(
    &self,
    parser: &mut JavascriptParser<'parser>,
    expr: &'ast VarDeclarator,
    stmt: &'ast VarDecl,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.declarator(parser, expr, stmt, self);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  pub fn new_expression(
    &self,
    parser: &mut JavascriptParser<'parser>,
    expr: &'ast swc_core::ecma::ast::NewExpr,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.new_expression(parser, expr, self);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }
}
