use rspack_error::{Error, Severity};
use rspack_util::swc::RspackComments;
use serde_json::json;
use swc_next_ecma_parser::{FragmentContext, NoTokenParserConfig, Options, Parser};

use super::BasicEvaluatedExpression;
use crate::visitors::{JavascriptParser, ParsedJavaScriptAst};

#[inline]
pub fn eval_source<'parser>(
  parser: &mut JavascriptParser<'parser>,
  source: String,
  error_title: String,
) -> Option<BasicEvaluatedExpression<'parser>> {
  // Keep the fragment and its source alive for any expression handles retained
  // by evaluation. Name lookup continues in the caller's scope.
  let allocator = parser.ast.ast.allocator();
  let source_in_allocator = allocator.alloc_str(&source);
  let result = Parser::init(
    allocator,
    source_in_allocator,
    Options {
      preserve_parens: false,
      ..Options::default()
    },
    NoTokenParserConfig,
  )
  .parse_expression_fragment(FragmentContext::TopLevel);
  match result {
    Err(diagnostics) => {
      let span = diagnostics
        .first()
        .map_or_default(|diagnostic| diagnostic.span);
      let mut error = Error::from_string(
        Some(source.clone()),
        span.start as usize,
        span.end as usize,
        format!("{error_title} warning"),
        format!("failed to parse {}", json!(source.as_str())),
      );
      error.severity = Severity::Warning;
      parser.add_warning(error.into());
      None
    }
    Ok(ast) => {
      let expression = ast.root_expression();
      let ast = allocator.alloc(ast);
      // SAFETY: Fragment handles stay with their owning AST, which is never mutated.
      let ast = unsafe { ast.ast() };
      let comments = allocator.alloc(RspackComments::from_ast(ast));
      let fragment = allocator.alloc(ParsedJavaScriptAst { ast, comments });
      Some(parser.evaluate_expression_in_ast(expression, fragment))
    }
  }
}
