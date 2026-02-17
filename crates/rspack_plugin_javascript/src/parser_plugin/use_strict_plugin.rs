use rspack_core::ConstDependency;
use swc_core::atoms::wtf8::Wtf8;
use swc_experimental_ecma_ast::{Lit, Program, Spanned};

use super::JavascriptParserPlugin;
use crate::visitors::JavascriptParser;

pub struct UseStrictPlugin;

impl JavascriptParserPlugin for UseStrictPlugin {
  fn program(&self, parser: &mut JavascriptParser, ast: Program) -> Option<bool> {
    let first = match ast {
      Program::Module(ast) => ast
        .body(&parser.ast)
        .first()
        .and_then(|i| parser.ast.get_node_in_sub_range(i).as_stmt()),
      Program::Script(ast) => ast
        .body(&parser.ast)
        .first()
        .map(|i| parser.ast.get_node_in_sub_range(i)),
    }
    .and_then(|i| i.as_expr());
    if let Some(first) = first
      && first.expr(&parser.ast).as_lit().and_then(|i| match i {
        Lit::Str(s) => Some(parser.ast.get_wtf8(s.value(&parser.ast))),
        _ => None,
      }) == Some(Wtf8::from_str("use strict"))
    {
      // Remove "use strict" expression. It will be added later by the renderer again.
      // This is necessary in order to not break the strict mode when webpack prepends code.
      let dep = ConstDependency::new(first.span(&parser.ast).into(), "".into());
      parser.add_presentational_dependency(Box::new(dep));
      parser.build_info.strict = true;
    }
    None
  }
}
