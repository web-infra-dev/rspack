use rspack_core::ConstDependency;
use swc_core::atoms::wtf8::Wtf8;

use super::JavascriptParserPlugin;
use crate::visitors::JavascriptParser;

pub(crate) struct UseStrictPlugin;

impl JavascriptParserPlugin for UseStrictPlugin {
  fn program(
    &self,
    parser: &mut JavascriptParser,
    ast: &swc_core::ecma::ast::Program,
  ) -> Option<bool> {
    let first = match ast {
      swc_core::ecma::ast::Program::Module(ast) => ast.body.first().and_then(|i| i.as_stmt()),
      swc_core::ecma::ast::Program::Script(ast) => ast.body.first(),
    }
    .and_then(|i| i.as_expr());
    if let Some(first) = first
      && first.expr.as_lit().and_then(|i| match i {
        swc_core::ecma::ast::Lit::Str(s) => Some(s.value.as_wtf8()),
        _ => None,
      }) == Some(Wtf8::from_str("use strict"))
    {
      // Remove "use strict" expression. It will be added later by the renderer again.
      // This is necessary in order to not break the strict mode when webpack prepends code.
      let dep = ConstDependency::new(first.span.into(), "".into());
      parser.add_presentational_dependency(Box::new(dep));
      parser.build_info.strict = true;
    }
    None
  }
}
