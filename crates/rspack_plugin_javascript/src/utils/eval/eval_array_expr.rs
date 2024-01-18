use rspack_core::SpanExt;
use swc_core::ecma::ast::ArrayLit;

use super::BasicEvaluatedExpression;
use crate::{parser_plugin::JavaScriptParserPluginDrive, visitors::JavascriptParser};

pub fn eval_array_expression<'ast, 'parser>(
  scanner: &mut JavascriptParser<'parser>,
  expr: &'ast ArrayLit,
  plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
) -> Option<BasicEvaluatedExpression<'ast>> {
  let mut items = vec![];

  for elem in &expr.elems {
    if let Some(elem) = elem
      && elem.spread.is_none()
    {
      items.push(scanner.evaluate_expression(&elem.expr, plugin_drive));
    } else {
      return None;
    }
  }

  let mut res = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.hi().0);
  res.set_items(items);
  Some(res)
}
