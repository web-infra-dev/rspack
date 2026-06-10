use rspack_core::ConstDependency;
use rspack_util::SpanExt;
use swc_experimental_ecma_ast::{CallExpr, UnaryExpr};

use super::JavascriptParserPlugin;
use crate::{dependency::IsIncludeDependency, visitors::JavascriptParser};

const IS_INCLUDED: &str = "__webpack_is_included__";

pub struct IsIncludedPlugin;

#[rspack_macros::implemented_javascript_parser_hooks]
impl<'p, 'a> JavascriptParserPlugin<'p, 'a> for IsIncludedPlugin {
  fn call(&self, parser: &mut JavascriptParser<'p>, expr: &CallExpr, name: &str) -> Option<bool> {
    if name != IS_INCLUDED || expr.args.len() != 1 || expr.args[0].spread.is_some() {
      return None;
    }

    let request = parser.evaluate_expression(&expr.args[0].expr);
    if !request.is_string() {
      return None;
    }

    parser.add_dependency(Box::new(IsIncludeDependency::new(
      (expr.span.real_lo(), expr.span.real_hi()).into(),
      request.string().clone(),
    )));

    Some(true)
  }

  fn r#typeof(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: &UnaryExpr,
    for_name: &str,
  ) -> Option<bool> {
    (for_name == IS_INCLUDED).then(|| {
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        (expr.span.real_lo(), expr.span.real_hi()).into(),
        "'function'".into(),
      )));
      true
    })
  }
}
