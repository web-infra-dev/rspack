use rspack_core::ConstDependency;
use rspack_util::SpanExt;
use swc_experimental_ecma_ast::{CallExpr, GetSpan, UnaryExpr};

use super::JavascriptParserPlugin;
use crate::{dependency::IsIncludeDependency, visitors::JavascriptParser};

const IS_INCLUDED: &str = "__webpack_is_included__";

pub struct IsIncludedPlugin;

impl JavascriptParserPlugin for IsIncludedPlugin {
  fn call(&self, parser: &mut JavascriptParser, expr: CallExpr, name: &str) -> Option<bool> {
    if name != IS_INCLUDED
      || expr.args(&parser.ast).len() != 1
      || parser
        .ast
        .get_node_in_sub_range(expr.args(&parser.ast).get(0).unwrap())
        .spread(&parser.ast)
        .is_some()
    {
      return None;
    }

    let request = parser.evaluate_expression(
      parser
        .ast
        .get_node_in_sub_range(expr.args(&parser.ast).get(0).unwrap())
        .expr(&parser.ast),
    );
    if !request.is_string() {
      return None;
    }

    parser.add_dependency(Box::new(IsIncludeDependency::new(
      (
        expr.span(&parser.ast).real_lo(),
        expr.span(&parser.ast).real_hi(),
      )
        .into(),
      request.string().clone(),
    )));

    Some(true)
  }

  fn r#typeof(
    &self,
    parser: &mut JavascriptParser<'_>,
    expr: UnaryExpr,
    for_name: &str,
  ) -> Option<bool> {
    (for_name == IS_INCLUDED).then(|| {
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        (
          expr.span(&parser.ast).real_lo(),
          expr.span(&parser.ast).real_hi(),
        )
          .into(),
        "'function'".into(),
      )));
      true
    })
  }
}
