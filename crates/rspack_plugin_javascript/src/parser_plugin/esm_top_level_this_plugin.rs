use rspack_core::ConstDependency;
use swc_experimental_ecma_ast::{GetSpan, ThisExpr};

use super::JavascriptParserPlugin;
use crate::visitors::JavascriptParser;

pub struct ESMTopLevelThisParserPlugin;

impl JavascriptParserPlugin for ESMTopLevelThisParserPlugin {
  fn this(&self, parser: &mut JavascriptParser, expr: ThisExpr, _for_name: &str) -> Option<bool> {
    (parser.is_esm && parser.is_top_level_this()).then(|| {
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        expr.span(&parser.ast).into(),
        "undefined".into(),
      )));
      true
    })
  }
}
