use std::sync::Arc;

use rspack_core::ConstDependency;
use rspack_plugin_javascript::{JavascriptParserPlugin, visitors::JavascriptParser};
use swc_next_ecma_ast::{GetSpan, Program};

pub struct ReactDirectivesParserPlugin;

#[rspack_plugin_javascript::implemented_javascript_parser_hooks]
impl<'p, 'a> JavascriptParserPlugin<'p, 'a> for ReactDirectivesParserPlugin {
  fn program(&self, parser: &mut JavascriptParser<'p>, program: Program) -> Option<bool> {
    let ast = parser.ast.ast;
    let directives = program
      .directives(ast)
      .iter()
      .map(|id| ast.get_node_in_sub_range(id))
      .take_while(|directive| ast.get_utf8(directive.value(ast)).starts_with("use "))
      .map(|directive| {
        (
          format!("\"{}\"", ast.get_utf8(directive.value(ast))),
          directive.span(ast),
        )
      })
      .collect::<Vec<_>>();

    if directives.is_empty() {
      return None;
    }

    parser.build_info.extras.insert(
      "react_directives".to_string(),
      serde_json::json!(directives.iter().map(|(d, _)| d).collect::<Vec<_>>()),
    );

    for (_, span) in directives {
      parser.add_presentational_dependency(Arc::new(ConstDependency::new(span.into(), "".into())));
    }

    None
  }
}
