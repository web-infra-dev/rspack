use swc_core::common::Span;
use swc_core::ecma::ast::{BinExpr, CallExpr, Callee, CondExpr};
use swc_core::ecma::ast::{ExportDecl, ExportDefaultDecl, Expr, Ident, OptChainExpr};
use swc_core::ecma::ast::{IfStmt, MemberExpr, Stmt, UnaryOp, VarDecl, VarDeclarator};

use super::{BoxJavascriptParserPlugin, JavascriptParserPlugin};
use crate::parser_plugin::r#const::is_logic_op;
use crate::utils::eval::BasicEvaluatedExpression;
use crate::visitors::{ExportedVariableInfo, JavascriptParser};

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

  fn finish(
    &self,
    parser: &mut JavascriptParser,
    ast: &swc_core::ecma::ast::Program,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.finish(parser, ast);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn pre_module_declaration(
    &self,
    parser: &mut JavascriptParser,
    decl: &swc_core::ecma::ast::ModuleDecl,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.pre_module_declaration(parser, decl);
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

  fn call_member_chain(
    &self,
    parser: &mut JavascriptParser,
    root_info: &ExportedVariableInfo,
    expr: &CallExpr,
    // TODO: members: &Vec<String>,
    // TODO: members_optionals: Vec<bool>,
    // TODO: members_ranges: Vec<DependencyLoc>
  ) -> Option<bool> {
    assert!(matches!(expr.callee, Callee::Expr(_)));
    for plugin in &self.plugins {
      let res = plugin.call_member_chain(parser, root_info, expr);
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
    for_name: &str,
  ) -> Option<bool> {
    assert!(expr.op == UnaryOp::TypeOf);
    for plugin in &self.plugins {
      let res = plugin.r#typeof(parser, expr, for_name);
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

  fn evaluate_call_expression_member(
    &self,
    parser: &mut JavascriptParser,
    property: &str,
    expr: &CallExpr,
    param: &BasicEvaluatedExpression,
  ) -> Option<BasicEvaluatedExpression> {
    for plugin in &self.plugins {
      let res = plugin.evaluate_call_expression_member(parser, property, expr, param);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn evaluate_identifier(
    &self,
    parser: &mut JavascriptParser,
    ident: &str,
    start: u32,
    end: u32,
  ) -> Option<BasicEvaluatedExpression> {
    for plugin in &self.plugins {
      let res = plugin.evaluate_identifier(parser, ident, start, end);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn can_rename(&self, parser: &mut JavascriptParser, str: &str) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.can_rename(parser, str);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn rename(&self, parser: &mut JavascriptParser, expr: &Expr, str: &str) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.rename(parser, expr, str);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn pre_statement(&self, parser: &mut JavascriptParser, stmt: &Stmt) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.pre_statement(parser, stmt);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn pre_declarator(&self, parser: &mut JavascriptParser, decl: &VarDeclarator) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.pre_declarator(parser, decl);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn block_pre_statement(&self, parser: &mut JavascriptParser, stmt: &Stmt) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.block_pre_statement(parser, stmt);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn block_pre_module_declration(
    &self,
    parser: &mut JavascriptParser,
    decl: &swc_core::ecma::ast::ModuleDecl,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.block_pre_module_declration(parser, decl);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn import_call(&self, parser: &mut JavascriptParser, expr: &CallExpr) -> Option<bool> {
    assert!(expr.callee.is_import());
    for plugin in &self.plugins {
      let res = plugin.import_call(parser, expr);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn meta_property(
    &self,
    parser: &mut JavascriptParser,
    root_name: &swc_core::atoms::Atom,
    span: Span,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.meta_property(parser, root_name, span);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn unhandled_expression_member_chain(
    &self,
    parser: &mut JavascriptParser,
    root_info: &ExportedVariableInfo,
    expr: &MemberExpr,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.unhandled_expression_member_chain(parser, root_info, expr);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn import(
    &self,
    parser: &mut JavascriptParser,
    statement: &swc_core::ecma::ast::ImportDecl,
    source: &str,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.import(parser, statement, source);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn import_specifier(
    &self,
    parser: &mut JavascriptParser,
    statement: &swc_core::ecma::ast::ImportDecl,
    source: &swc_core::atoms::Atom,
    export_name: Option<&str>,
    identifier_name: &str,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.import_specifier(parser, statement, source, export_name, identifier_name);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn named_export_import(
    &self,
    parser: &mut JavascriptParser,
    statement: &swc_core::ecma::ast::NamedExport,
    source: &str,
  ) -> Option<bool> {
    assert!(statement.src.is_some());
    for plugin in &self.plugins {
      let res = plugin.named_export_import(parser, statement, source);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn all_export_import(
    &self,
    parser: &mut JavascriptParser,
    statement: &swc_core::ecma::ast::ExportAll,
    source: &str,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.all_export_import(parser, statement, source);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn optional_chaining(&self, parser: &mut JavascriptParser, expr: &OptChainExpr) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.optional_chaining(parser, expr);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn expression_conditional_operation(
    &self,
    parser: &mut JavascriptParser,
    expr: &CondExpr,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.expression_conditional_operation(parser, expr);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn export(&self, parser: &mut JavascriptParser, expr: &ExportDefaultDecl) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.export(parser, expr);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn export_default_expr(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::ExportDefaultExpr,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.export_default_expr(parser, expr);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn export_decl(&self, parser: &mut JavascriptParser, expr: &ExportDecl) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.export_decl(parser, expr);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn named_export(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::NamedExport,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.named_export(parser, expr);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn statement(&self, parser: &mut JavascriptParser, stmt: &Stmt) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.statement(parser, stmt);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn module_declaration(
    &self,
    parser: &mut JavascriptParser,
    decl: &swc_core::ecma::ast::ModuleDecl,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.module_declaration(parser, decl);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn class_extends_expression(
    &self,
    parser: &mut JavascriptParser,
    super_class: &Expr,
    classy: &swc_core::ecma::ast::Class,
    ident: Option<&Ident>,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.class_extends_expression(parser, super_class, classy, ident);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn class_body_element(
    &self,
    parser: &mut JavascriptParser,
    element: &swc_core::ecma::ast::ClassMember,
    classy: &swc_core::ecma::ast::Class,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.class_body_element(parser, element, classy);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn class_body_body(
    &self,
    parser: &mut JavascriptParser,
    body: &swc_core::ecma::ast::BlockStmt,
    element: &swc_core::ecma::ast::ClassMember,
    classy: &swc_core::ecma::ast::Class,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.class_body_body(parser, body, element, classy);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn class_body_value(
    &self,
    parser: &mut JavascriptParser,
    value: &swc_core::ecma::ast::Expr,
    element: &swc_core::ecma::ast::ClassMember,
    classy: &swc_core::ecma::ast::Class,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.class_body_value(parser, value, element, classy);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn is_pure_identifier(&self, parser: &mut JavascriptParser, ident: &Ident) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.is_pure_identifier(parser, ident);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }

  fn block_pre_walk_export_default_declaration(
    &self,
    parser: &mut JavascriptParser,
    decl: &ExportDefaultDecl,
  ) -> Option<bool> {
    for plugin in &self.plugins {
      let res = plugin.block_pre_walk_export_default_declaration(parser, decl);
      // `SyncBailHook`
      if res.is_some() {
        return res;
      }
    }
    None
  }
}
