use rspack_core::{ConstDependency, SpanExt};

use super::JavascriptParserPlugin;
use crate::visitors::{JavascriptParser, TopLevelScope};

pub struct ESMTopLevelThisParserPlugin;

impl JavascriptParserPlugin for ESMTopLevelThisParserPlugin {
  fn this(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::ThisExpr,
  ) -> Option<bool> {
    (parser.is_esm && !matches!(parser.top_level_scope, TopLevelScope::False)).then(|| {
      // TODO: esm_export::is_enabled
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
