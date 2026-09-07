use std::sync::Arc;

use rspack_core::ConstDependency;
use swc_next_ecma_ast::{GetSpan, ThisExpression};

use super::JavascriptParserPlugin;
use crate::visitors::JavascriptParser;

pub struct ESMTopLevelThisParserPlugin;

#[rspack_macros::implemented_javascript_parser_hooks]
impl<'p, 'a> JavascriptParserPlugin<'p, 'a> for ESMTopLevelThisParserPlugin {
  fn this(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: ThisExpression,
    _for_name: &str,
  ) -> Option<bool> {
    (parser.is_esm && parser.is_top_level_this()).then(|| {
      parser.add_presentational_dependency(Arc::new(ConstDependency::new(
        expr.span(parser.ast.ast).into(),
        "undefined".into(),
      )));
      true
    })
  }
}
