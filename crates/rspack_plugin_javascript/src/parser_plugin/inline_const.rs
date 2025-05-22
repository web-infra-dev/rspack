use swc_core::ecma::ast::Program;

use super::JavascriptParserPlugin;
use crate::visitors::JavascriptParser;

#[derive(Default)]
pub struct InlineConstPlugin;

impl JavascriptParserPlugin for InlineConstPlugin {
  fn program(&self, parser: &mut JavascriptParser, _: &Program) -> Option<bool> {
    None
  }
}
