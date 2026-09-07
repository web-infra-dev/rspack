use std::sync::Arc;

use rspack_core::ConstDependency;
use rspack_plugin_javascript::{JavascriptParserPlugin, visitors::JavascriptParser};
use rspack_util::swc::AstSubRangeExt;
use swc_next_ecma_ast::{GetSpan, Program};

pub struct ReactDirectivesParserPlugin;

#[rspack_plugin_javascript::implemented_javascript_parser_hooks]
impl<'p, 'a> JavascriptParserPlugin<'p, 'a> for ReactDirectivesParserPlugin {
  fn program(&self, parser: &mut JavascriptParser<'p>, program: Program) -> Option<bool> {
    let ast = parser.ast.ast;
    let directives = program.directives(ast);
    let values: Vec<_> = ast
      .nodes(directives)
      .take_while(|directive| ast.get_utf8(directive.value(ast)).starts_with("use "))
      .map(|directive| {
        serde_json::Value::String(format!("\"{}\"", ast.get_utf8(directive.value(ast))))
      })
      .collect();

    if values.is_empty() {
      return None;
    }

    let directive_count = values.len();
    parser.build_info.extras.insert(
      "react_directives".to_string(),
      serde_json::Value::Array(values),
    );

    for directive in ast.nodes(directives).take(directive_count) {
      parser.add_presentational_dependency(Arc::new(ConstDependency::new(
        directive.span(ast).into(),
        "".into(),
      )));
    }

    None
  }
}
