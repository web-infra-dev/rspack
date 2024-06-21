use rspack_core::SpanExt;

use super::JavascriptParserPlugin;
use crate::{dependency::URLDependency, visitors::JavascriptParser};

pub struct URLPlugin {
  pub relative: bool,
}

impl JavascriptParserPlugin for URLPlugin {
  fn new_expression(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::NewExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == "URL"
      && let Some((start, end, request)) = rspack_core::needs_refactor::match_new_url(expr)
    {
      parser.dependencies.push(Box::new(URLDependency::new(
        start,
        end,
        expr.span.real_lo(),
        expr.span.real_hi(),
        request.into(),
        Some(expr.span.into()),
        self.relative,
      )));
      Some(true)
    } else {
      None
    }
  }
}
