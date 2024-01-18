use swc_core::ecma::ast::{BinExpr, CallExpr, Ident, IfStmt, MemberExpr, NewExpr, UnaryExpr};
use swc_core::ecma::ast::{VarDecl, VarDeclarator};

use super::JavaScriptParserPluginDrive;
use crate::utils::eval::BasicEvaluatedExpression;
use crate::visitors::JavascriptParser;

type KeepRight = bool;

pub trait JavascriptParserPlugin<'ast, 'parser> {
  fn evaluate_typeof(
    &self,
    _parser: &mut JavascriptParser<'parser>,
    _ident: &'ast Ident,
    _start: u32,
    _end: u32,
    _plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) -> Option<BasicEvaluatedExpression<'ast>> {
    None
  }

  fn call(
    &self,
    _parser: &mut JavascriptParser<'parser>,
    _expr: &'ast CallExpr,
    _plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) -> Option<bool> {
    None
  }

  // FIXME: should remove
  fn member(
    &self,
    _parser: &mut JavascriptParser<'parser>,
    _expr: &'ast MemberExpr,
    _plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) -> Option<bool> {
    None
  }

  fn r#typeof(
    &self,
    _parser: &mut JavascriptParser<'parser>,
    _expr: &'ast UnaryExpr,
    _plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) -> Option<bool> {
    None
  }

  /// Return:
  /// - `None` means should walk left and right;
  /// - `Some(true)` means should walk right;
  /// - `Some(false)` means nothing need to do.
  fn expression_logical_operator(
    &self,
    _parser: &mut JavascriptParser<'parser>,
    _expr: &'ast BinExpr,
    _plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) -> Option<KeepRight> {
    None
  }

  /// Return:
  /// - `None` means should walk left and right;
  fn binary_expression(
    &self,
    _parser: &mut JavascriptParser<'parser>,
    _expr: &'ast BinExpr,
    _plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) -> Option<KeepRight> {
    None
  }

  /// Return:
  /// - `None` means need walk `stmt.test`, `stmt.cons` and `stmt.alt`;
  /// - `Some(true)` means only need walk `stmt.cons`;
  /// - `Some(false)` means only need walk `stmt.alt`;
  fn statement_if(
    &self,
    _parser: &mut JavascriptParser<'parser>,
    _expr: &'ast IfStmt,
    _plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) -> Option<bool> {
    None
  }

  fn declarator(
    &self,
    _parser: &mut JavascriptParser<'parser>,
    _expr: &'ast VarDeclarator,
    _stmt: &'ast VarDecl,
    _plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) -> Option<bool> {
    None
  }

  fn new_expression(
    &self,
    _parser: &mut JavascriptParser<'parser>,
    _expr: &'ast NewExpr,
    _plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) -> Option<bool> {
    None
  }
}

pub type BoxJavascriptParserPlugin<'ast, 'parser> = Box<dyn JavascriptParserPlugin<'ast, 'parser>>;
