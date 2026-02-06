use rspack_core::ConstDependency;

use super::JavascriptParserPlugin;
use crate::visitors::JavascriptParser;

pub(crate) struct ESMTopLevelThisParserPlugin;

impl JavascriptParserPlugin for ESMTopLevelThisParserPlugin {
  fn this(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::ThisExpr,
    _for_name: &str,
  ) -> Option<bool> {
    (parser.is_esm && parser.is_top_level_this()).then(|| {
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        expr.span.into(),
        "undefined".into(),
      )));
      true
    })
  }
}
