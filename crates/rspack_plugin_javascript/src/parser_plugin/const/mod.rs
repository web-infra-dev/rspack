mod if_stmt;
mod logic_expr;

pub use self::logic_expr::is_logic_op;
use super::JavascriptParserPlugin;
use crate::visitors::JavascriptParser;

pub struct ConstPlugin;

impl JavascriptParserPlugin for ConstPlugin {
  fn expression_logical_operator(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::BinExpr,
  ) -> Option<bool> {
    self::logic_expr::expression_logic_operator(parser, expr)
  }

  fn statement_if(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::IfStmt,
  ) -> Option<bool> {
    self::if_stmt::statement_if(parser, expr)
  }
}
