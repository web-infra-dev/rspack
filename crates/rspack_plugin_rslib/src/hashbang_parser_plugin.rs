use std::sync::Arc;

use rspack_core::ConstDependency;
use rspack_plugin_javascript::{JavascriptParserPlugin, visitors::JavascriptParser};
use swc_next_ecma_ast::{GetSpan, Program};

pub struct HashbangParserPlugin;

#[rspack_plugin_javascript::implemented_javascript_parser_hooks]
impl<'p, 'a> JavascriptParserPlugin<'p, 'a> for HashbangParserPlugin {
  fn program(&self, parser: &mut JavascriptParser<'p>, program: Program) -> Option<bool> {
    let ast = parser.ast.ast;
    let hashbang = program.hashbang(ast)?;
    let hashbang_value = ast.get_utf8(hashbang.value(ast));

    // SWC Next stores the hashbang value without the leading "#!".
    let normalized_hashbang = format!("#!{hashbang_value}");

    // Store hashbang in build_info for later use during rendering
    parser.build_info.extras.insert(
      "hashbang".to_string(),
      serde_json::Value::String(normalized_hashbang),
    );

    parser.add_presentational_dependency(Arc::new(ConstDependency::new(
      hashbang.span(ast).into(),
      "".into(),
    )));

    None
  }
}
