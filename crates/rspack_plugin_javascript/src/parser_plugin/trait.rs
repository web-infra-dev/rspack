use swc_core::atoms::Atom;
use swc_core::common::Span;
use swc_core::ecma::ast::{
  AssignExpr, AwaitExpr, BinExpr, CallExpr, CondExpr, ExportAll, ExportDecl, ExportDefaultDecl,
  ExportDefaultExpr, Expr, ForOfStmt, Ident, IfStmt, ImportDecl, MemberExpr, ModuleDecl,
  NamedExport, OptChainExpr,
};
use swc_core::ecma::ast::{NewExpr, Program, Stmt, ThisExpr, UnaryExpr, VarDecl, VarDeclarator};

use crate::utils::eval::BasicEvaluatedExpression;
use crate::visitors::{ExportedVariableInfo, JavascriptParser};

type KeepRight = bool;

pub trait JavascriptParserPlugin {
  /// Return:
  /// - `Some(true)` signifies the termination of the current
  /// statement's visit during the pre-walk phase.
  /// - Other return values imply that the walk operation ought to continue
  fn pre_statement(&self, _parser: &mut JavascriptParser, _stmt: &Stmt) -> Option<bool> {
    None
  }

  /// The return value will have no effect.
  fn top_level_await_expr(&self, _parser: &mut JavascriptParser, _expr: &AwaitExpr) {}

  /// The return value will have no effect.
  fn top_level_for_of_await_stmt(&self, _parser: &mut JavascriptParser, _stmt: &ForOfStmt) {}

  fn can_rename(&self, _parser: &mut JavascriptParser, _str: &str) -> Option<bool> {
    None
  }

  fn rename(&self, _parser: &mut JavascriptParser, _expr: &Expr, _str: &str) -> Option<bool> {
    None
  }

  fn program(&self, _parser: &mut JavascriptParser, _ast: &Program) -> Option<bool> {
    None
  }

  /// Return:
  /// `None` means continue this `ModuleDecl`
  /// Others means skip this.
  ///
  /// This is similar `hooks.pre_statement` in webpack
  fn pre_module_declaration(
    &self,
    _parser: &mut JavascriptParser,
    _decl: &ModuleDecl,
  ) -> Option<bool> {
    None
  }

  fn pre_declarator(
    &self,
    _parser: &mut JavascriptParser,
    _declarator: &VarDeclarator,
    _declaration: &VarDecl,
  ) -> Option<bool> {
    None
  }

  fn evaluate_typeof(
    &self,
    _parser: &mut JavascriptParser,
    _expr: &UnaryExpr,
    _for_name: &str,
  ) -> Option<BasicEvaluatedExpression> {
    None
  }

  fn evaluate_identifier(
    &self,
    _parser: &mut JavascriptParser,
    _ident: &str,
    _start: u32,
    _end: u32,
  ) -> Option<BasicEvaluatedExpression> {
    None
  }

  fn evaluate_call_expression_member(
    &self,
    _parser: &mut JavascriptParser,
    _property: &str,
    _expr: &CallExpr,
    _param: &BasicEvaluatedExpression,
  ) -> Option<BasicEvaluatedExpression> {
    None
  }

  fn pattern(
    &self,
    _parser: &mut JavascriptParser,
    _ident: &Ident,
    _for_name: &str,
  ) -> Option<bool> {
    None
  }

  fn call(
    &self,
    _parser: &mut JavascriptParser,
    _expr: &CallExpr,
    _for_name: &str,
  ) -> Option<bool> {
    None
  }

  fn call_member_chain(
    &self,
    _parser: &mut JavascriptParser,
    _expr: &CallExpr,
    _for_name: &str,
    _members: &[Atom],
    _members_optionals: &[bool],
    _member_ranges: &[Span],
  ) -> Option<bool> {
    None
  }

  fn member(
    &self,
    _parser: &mut JavascriptParser,
    _expr: &MemberExpr,
    _for_name: &str,
  ) -> Option<bool> {
    None
  }

  fn member_chain(
    &self,
    _parser: &mut JavascriptParser,
    _expr: &MemberExpr,
    _for_name: &str,
    _members: &[Atom],
    _members_optionals: &[bool],
    _member_ranges: &[Span],
  ) -> Option<bool> {
    None
  }

  fn unhandled_expression_member_chain(
    &self,
    _parser: &mut JavascriptParser,
    _root_info: &ExportedVariableInfo,
    _expr: &MemberExpr,
  ) -> Option<bool> {
    None
  }

  fn member_chain_of_call_member_chain(
    &self,
    _parser: &mut JavascriptParser,
    _expr: &MemberExpr,
    _for_name: &str,
  ) -> Option<bool> {
    None
  }

  fn call_member_chain_of_call_member_chain(
    &self,
    _parser: &mut JavascriptParser,
    _expr: &CallExpr,
    _for_name: &str,
  ) -> Option<bool> {
    None
  }

  fn r#typeof(
    &self,
    _parser: &mut JavascriptParser,
    _expr: &UnaryExpr,
    _for_name: &str,
  ) -> Option<bool> {
    None
  }

  /// Return:
  /// - `None` means should walk left and right;
  /// - `Some(true)` means should walk right;
  /// - `Some(false)` means nothing need to do.
  fn expression_logical_operator(
    &self,
    _parser: &mut JavascriptParser,
    _expr: &BinExpr,
  ) -> Option<KeepRight> {
    None
  }

  /// Return:
  /// - `None` means should walk left and right;
  fn binary_expression(
    &self,
    _parser: &mut JavascriptParser,
    _expr: &BinExpr,
  ) -> Option<KeepRight> {
    None
  }

  /// Return:
  /// - `None` means need walk `stmt.test`, `stmt.cons` and `stmt.alt`;
  /// - `Some(true)` means only need walk `stmt.cons`;
  /// - `Some(false)` means only need walk `stmt.alt`;
  fn statement_if(&self, _parser: &mut JavascriptParser, _expr: &IfStmt) -> Option<bool> {
    None
  }

  fn declarator(
    &self,
    _parser: &mut JavascriptParser,
    _expr: &VarDeclarator,
    _stmt: &VarDecl,
  ) -> Option<bool> {
    None
  }

  fn new_expression(
    &self,
    _parser: &mut JavascriptParser,
    _expr: &NewExpr,
    _for_name: &str,
  ) -> Option<bool> {
    None
  }

  fn identifier(
    &self,
    _parser: &mut JavascriptParser,
    _ident: &Ident,
    _for_name: &str,
  ) -> Option<bool> {
    None
  }

  fn this(&self, _parser: &mut JavascriptParser, _expr: &ThisExpr) -> Option<bool> {
    None
  }

  // FIXME: should remove
  fn assign(&self, _parser: &mut JavascriptParser, _expr: &AssignExpr) -> Option<bool> {
    None
  }

  fn import_call(&self, _parser: &mut JavascriptParser, _expr: &CallExpr) -> Option<bool> {
    None
  }

  fn meta_property(
    &self,
    _parser: &mut JavascriptParser,
    _root_name: &Atom,
    _span: Span,
  ) -> Option<bool> {
    None
  }

  fn import(
    &self,
    _parser: &mut JavascriptParser,
    _statement: &ImportDecl,
    _source: &str,
  ) -> Option<bool> {
    None
  }

  fn import_specifier(
    &self,
    _parser: &mut JavascriptParser,
    _statement: &ImportDecl,
    _source: &Atom,
    _export_name: Option<&Atom>,
    _identifier_name: &Atom,
  ) -> Option<bool> {
    None
  }

  fn named_export_import(
    &self,
    _parser: &mut JavascriptParser,
    _statement: &NamedExport,
    _source: &str,
  ) -> Option<bool> {
    None
  }

  fn all_export_import(
    &self,
    _parser: &mut JavascriptParser,
    _statement: &ExportAll,
    _source: &str,
  ) -> Option<bool> {
    None
  }

  fn optional_chaining(
    &self,
    _parser: &mut JavascriptParser,
    _expr: &OptChainExpr,
  ) -> Option<bool> {
    None
  }

  fn expression_conditional_operation(
    &self,
    _parser: &mut JavascriptParser,
    _expr: &CondExpr,
  ) -> Option<bool> {
    None
  }

  fn export(&self, _parser: &mut JavascriptParser, _expr: &ExportDefaultDecl) -> Option<bool> {
    None
  }

  // TODO: remove `export_default_expr`
  fn export_default_expr(
    &self,
    _parser: &mut JavascriptParser,
    _expr: &ExportDefaultExpr,
  ) -> Option<bool> {
    None
  }

  // TODO: remove `export_decl`
  fn export_decl(&self, _parser: &mut JavascriptParser, _expr: &ExportDecl) -> Option<bool> {
    None
  }

  // TODO: remove `named_export`
  fn named_export(&self, _parser: &mut JavascriptParser, _expr: &NamedExport) -> Option<bool> {
    None
  }

  fn finish(&self, _parser: &mut JavascriptParser) -> Option<bool> {
    None
  }
}

pub type BoxJavascriptParserPlugin = Box<dyn JavascriptParserPlugin>;
