use swc_core::ecma::ast::{BinExpr, CallExpr, IfStmt, VarDecl, VarDeclarator};

use super::{BoxJavascriptParserPlugin, JavascriptParserPlugin};
use crate::parser_plugin::r#const::is_logic_op;
use crate::utils::eval::BasicEvaluatedExpression;
use crate::visitors::JavascriptParser;

pub struct JavaScriptParserPluginDrive {
  plugins: Vec<BoxJavascriptParserPlugin>,
}

impl JavaScriptParserPluginDrive {
  pub fn new(plugins: Vec<BoxJavascriptParserPlugin>) -> Self {
    Self { plugins }
  }
}

impl JavascriptParserPlugin for JavaScriptParserPluginDrive {
  fn top_level_await_expr(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::AwaitExpr,
  ) {
    for plugin in &self.plugins {
      // `SyncBailHook` but without return value
      plugin.top_level_await_expr(parser, expr);
    }
  }

  fn top_level_for_of_await_stmt(
    &self,
    parser: &mut JavascriptParser,
    stmt: &swc_core::ecma::ast::ForOfStmt,
  ) {
    for plugin in &self.plugins {
      // `SyncBailHook` but without return value
      plugin.top_level_for_of_await_stmt(parser, stmt);
    }
  }

  fn program(
    &self,
    parser: &mut JavascriptParser,
    ast: &swc_core::ecma::ast::Program,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.program(parser, ast);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn evaluate_typeof(
    &self,
    parser: &mut JavascriptParser,
    ident: &swc_core::ecma::ast::Ident,
    start: u32,
    end: u32,
  ) -> Option<BasicEvaluatedExpression> {
    for plugin in &self.plugins {
      let res = plugin.evaluate_typeof(parser, ident, start, end);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn call(&self, parser: &mut JavascriptParser, expr: &CallExpr, name: &str) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.call(parser, expr, name);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn member(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::MemberExpr,
    for_name: &str,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.member(parser, expr, for_name);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn member_chain_of_call_member_chain(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::MemberExpr,
    for_name: &str,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.member_chain_of_call_member_chain(parser, expr, for_name);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn call_member_chain_of_call_member_chain(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.call_member_chain_of_call_member_chain(parser, expr, for_name);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn assign(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::AssignExpr,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.assign(parser, expr);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn r#typeof(
    &self,
    parser: &mut JavascriptParser,
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

  fn expression_logical_operator(
    &self,
    parser: &mut JavascriptParser,
    expr: &BinExpr,
  ) -> Option<bool> {
    assert!(is_logic_op(expr.op));
    for plugin in &self.plugins {
      let res = plugin.expression_logical_operator(parser, expr);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn binary_expression(&self, parser: &mut JavascriptParser, expr: &BinExpr) -> Option<bool> {
    assert!(!is_logic_op(expr.op));
    for plugin in &self.plugins {
      let res = plugin.binary_expression(parser, expr);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn statement_if(&self, parser: &mut JavascriptParser, expr: &IfStmt) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.statement_if(parser, expr);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn declarator(
    &self,
    parser: &mut JavascriptParser,
    expr: &VarDeclarator,
    stmt: &VarDecl,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.declarator(parser, expr, stmt);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn new_expression(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::NewExpr,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.new_expression(parser, expr);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn identifier(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::Ident,
    for_name: &str,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.identifier(parser, expr, for_name);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn this(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::ThisExpr,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.this(parser, expr);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }
}
