use rspack_core::OverrideStrict;
use swc_core::ecma::ast::Program;

use super::JavascriptParserPlugin;
use crate::visitors::JavascriptParser;

#[derive(Default)]
pub struct OverrideStrictPlugin;

impl JavascriptParserPlugin for OverrideStrictPlugin {
  fn program(&self, parser: &mut JavascriptParser, _: &Program) -> Option<bool> {
    if let Some(strict) = parser.javascript_options.override_strict {
      parser.build_info.strict = matches!(strict, OverrideStrict::Strict);
    }

    None
  }
}
