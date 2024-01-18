mod if_stmt;
mod logic_expr;

pub use self::logic_expr::is_logic_op;
use super::JavascriptParserPlugin;
use crate::visitors::JavascriptParser;

pub struct ConstPlugin;

impl<'ast, 'parser> JavascriptParserPlugin<'ast, 'parser> for ConstPlugin {
  fn expression_logical_operator(
    &self,
    parser: &mut JavascriptParser<'parser>,
    expr: &'ast swc_core::ecma::ast::BinExpr,
    plugin_drive: &super::JavaScriptParserPluginDrive<'ast, 'parser>,
  ) -> Option<bool> {
    self::logic_expr::expression_logic_operator(parser, expr, plugin_drive)
  }

  fn statement_if(
    &self,
    parser: &mut JavascriptParser<'parser>,
    expr: &'ast swc_core::ecma::ast::IfStmt,
    plugin_drive: &super::JavaScriptParserPluginDrive<'ast, 'parser>,
  ) -> Option<bool> {
    self::if_stmt::statement_if(parser, expr, plugin_drive)
  }
}
