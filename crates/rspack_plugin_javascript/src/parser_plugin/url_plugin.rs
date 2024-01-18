use rspack_core::SpanExt;

use super::{JavaScriptParserPluginDrive, JavascriptParserPlugin};
use crate::dependency::URLDependency;

pub struct URLPlugin {
  pub relative: bool,
}

impl<'ast, 'parser> JavascriptParserPlugin<'ast, 'parser> for URLPlugin {
  fn new_expression(
    &self,
    parser: &mut crate::visitors::JavascriptParser<'parser>,
    expr: &'ast swc_core::ecma::ast::NewExpr,
    plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
  ) -> Option<bool> {
    if let Some(args) = &expr.args
      && parser.worker_syntax_list.match_new_worker(expr)
    {
      for arg in args.iter().skip(1) {
        parser.walk_expression(&arg.expr, plugin_drive);
      }
      // skip `new Worker(new Url,)`
      Some(true)
    } else if let Some((start, end, request)) = rspack_core::needs_refactor::match_new_url(expr) {
      parser.dependencies.push(Box::new(URLDependency::new(
        start,
        end,
        expr.span.real_lo(),
        expr.span.hi().0,
        request.into(),
        Some(expr.span.into()),
        self.relative,
      )));
      Some(true)
    } else {
      None
    }
  }
}
