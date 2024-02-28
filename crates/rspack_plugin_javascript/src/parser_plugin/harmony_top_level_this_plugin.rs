use rspack_core::{ConstDependency, SpanExt};

use super::JavascriptParserPlugin;
use crate::visitors::JavascriptParser;

pub struct HarmonyTopLevelThisParserPlugin;

impl JavascriptParserPlugin for HarmonyTopLevelThisParserPlugin {
  fn this(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::ThisExpr,
  ) -> Option<bool> {
    (parser.is_esm && parser.top_level_scope.top_or_top_arrow()).then(|| {
      // TODO: harmony_export::is_enabled
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          expr.span.real_lo(),
          expr.span.real_hi(),
          "undefined".into(),
          None,
        )));
      true
    })
  }
}
