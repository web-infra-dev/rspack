use rspack_core::ConstDependency;
use swc_next_ecma_ast::{GetSpan, Program};

use super::JavascriptParserPlugin;
use crate::visitors::JavascriptParser;

pub struct UseStrictPlugin;

#[rspack_macros::implemented_javascript_parser_hooks]
impl<'p, 'a> JavascriptParserPlugin<'p, 'a> for UseStrictPlugin {
  fn program(&self, parser: &mut JavascriptParser<'p>, program: Program) -> Option<bool> {
    let ast = parser.ast.ast;
    if let Some(first) = program
      .directives(ast)
      .iter()
      .next()
      .map(|id| ast.get_node_in_sub_range(id))
      && ast.get_utf8(first.value(ast)) == "use strict"
    {
      // Remove "use strict" expression. It will be added later by the renderer again.
      // This is necessary in order to not break the strict mode when webpack prepends code.
      // Directive.span includes its terminating semicolon, while the nested
      // string literal span does not. The legacy ExpressionStatement range
      // covered both and must be preserved for byte-for-byte replacement.
      let dep = ConstDependency::new(first.span(ast).into(), "".into());
      parser.add_presentational_dependency(Arc::new(dep));
      parser.build_info.strict = true;
    }
    None
  }
}
use std::sync::Arc;
