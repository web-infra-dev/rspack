use rspack_core::{ConstDependency, SpanExt};

use super::JavascriptParserPlugin;
use crate::visitors::JavascriptParser;

pub struct UseStrictPlugin;

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
      && matches!(
        first.expr.as_lit().and_then(|i| match i {
          swc_core::ecma::ast::Lit::Str(s) => Some(s.value.as_str()),
          _ => None,
        }),
        Some("use strict")
      )
    {
      // Remove "use strict" expression. It will be added later by the renderer again.
      // This is necessary in order to not break the strict mode when webpack prepends code.
      let dep = ConstDependency::new(first.span.real_lo(), first.span.real_hi(), "".into(), None);
      parser.presentational_dependencies.push(Box::new(dep));
      parser.build_info.strict = true;
    }
    None
  }
}
