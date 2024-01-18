use rspack_core::SpanExt;
use swc_core::ecma::ast::{UnaryExpr, UnaryOp};

use super::BasicEvaluatedExpression;
use crate::{parser_plugin::JavaScriptParserPluginDrive, visitors::JavascriptParser};

fn eval_typeof<'ast, 'parser>(
  scanner: &mut JavascriptParser<'parser>,
  expr: &'ast UnaryExpr,
  plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
) -> Option<BasicEvaluatedExpression<'ast>> {
  assert!(expr.op == UnaryOp::TypeOf);
  if let Some(ident) = expr.arg.as_ident()
    && let res = plugin_drive.evaluate_typeof(scanner, ident, expr.span.real_lo(), expr.span.hi().0)
    && res.is_some()
  {
    return res;
  }

  // TODO: if let `MetaProperty`, `MemberExpression` ...
  None
}

pub fn eval_unary_expression<'ast, 'parser>(
  scanner: &mut JavascriptParser<'parser>,
  expr: &'ast UnaryExpr,
  plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
) -> Option<BasicEvaluatedExpression<'ast>> {
  match expr.op {
    UnaryOp::TypeOf => eval_typeof(scanner, expr, plugin_drive),
    _ => None,
  }
}
