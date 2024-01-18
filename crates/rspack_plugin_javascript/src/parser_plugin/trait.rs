use swc_core::ecma::ast::{BinExpr, CallExpr, Ident, IfStmt, MemberExpr, NewExpr, UnaryExpr};
use swc_core::ecma::ast::{VarDecl, VarDeclarator};

use crate::utils::eval::BasicEvaluatedExpression;
use crate::visitors::JavascriptParser;

type KeepRight = bool;

pub trait JavascriptParserPlugin {
  fn evaluate_typeof(
    &self,
    _parser: &mut JavascriptParser,
    _ident: &Ident,
    _start: u32,
    _end: u32,
  ) -> Option<BasicEvaluatedExpression> {
    None
  }

  fn call(&self, _parser: &mut JavascriptParser, _expr: &CallExpr) -> Option<bool> {
    None
  }

  // FIXME: should remove
  fn member(&self, _parser: &mut JavascriptParser, _expr: &MemberExpr) -> Option<bool> {
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

  fn identifier(&self, _parser: &mut JavascriptParser, _expr: &Ident) -> Option<bool> {
    None
  }
}

pub type BoxJavascriptParserPlugin = Box<dyn JavascriptParserPlugin>;
