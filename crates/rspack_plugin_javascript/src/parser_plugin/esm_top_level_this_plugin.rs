use rspack_core::ConstDependency;

use super::JavascriptParserPlugin;
use crate::visitors::JavascriptParser;

pub struct ESMTopLevelThisParserPlugin;

impl JavascriptParserPlugin for ESMTopLevelThisParserPlugin {
  fn this(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::ThisExpr,
  ) -> Option<bool> {
    (parser.is_esm && parser.is_top_level_this()).then(|| {
      // TODO: esm_export::is_enabled
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          expr.span.into(),
          "undefined".into(),
          None,
        )));
      true
    })
  }
}
