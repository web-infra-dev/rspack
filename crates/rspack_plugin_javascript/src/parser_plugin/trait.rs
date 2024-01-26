use swc_core::ecma::ast::{
  AssignExpr, BinExpr, CallExpr, Ident, IfStmt, MemberExpr, NewExpr, Stmt, ThisExpr, UnaryExpr,
};
use swc_core::ecma::ast::{VarDecl, VarDeclarator};

use crate::utils::eval::BasicEvaluatedExpression;
use crate::visitors::JavascriptParser;

type KeepRight = bool;

pub trait JavascriptParserPlugin {
  /// Return:
  /// - `Some(true)` signifies the termination of the current
  /// statement's visit during the pre-walk phase.
  /// - Other return values imply that the walk operation ought to continue
  fn pre_statement(&self, _parser: &mut JavascriptParser, _stmt: &Stmt) -> Option<bool> {
    None
  }

  fn evaluate_typeof(
    &self,
    _parser: &mut JavascriptParser,
    _ident: &Ident,
    _start: u32,
    _end: u32,
  ) -> Option<BasicEvaluatedExpression> {
    None
  }

  fn call(&self, _parser: &mut JavascriptParser, _expr: &CallExpr, _name: &str) -> Option<bool> {
    None
  }

  fn member(
    &self,
    _parser: &mut JavascriptParser,
    _expr: &MemberExpr,
    _name: &str,
  ) -> Option<bool> {
    None
  }

  fn member_chain_of_call_member_chain(
    &self,
    _parser: &mut JavascriptParser,
    _expr: &MemberExpr,
  ) -> Option<bool> {
    None
  }

  fn r#typeof(&self, _parser: &mut JavascriptParser, _expr: &UnaryExpr) -> Option<bool> {
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

  fn new_expression(&self, _parser: &mut JavascriptParser, _expr: &NewExpr) -> Option<bool> {
    None
  }

  fn identifier(&self, _parser: &mut JavascriptParser, _ident: &Ident) -> Option<bool> {
    None
  }

  fn this(&self, _parser: &mut JavascriptParser, _expr: &ThisExpr) -> Option<bool> {
    None
  }

  // FIXME: should remove
  fn assign(&self, _parser: &mut JavascriptParser, _expr: &AssignExpr) -> Option<bool> {
    None
  }
}

pub type BoxJavascriptParserPlugin = Box<dyn JavascriptParserPlugin>;
