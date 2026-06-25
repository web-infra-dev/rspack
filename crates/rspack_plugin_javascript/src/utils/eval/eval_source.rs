use rspack_error::{Error, Severity};
use serde_json::json;
use swc_experimental_ecma_ast::EsVersion;
use swc_experimental_ecma_parser::{EsSyntax, Syntax, parse_file_as_expr};
use swc_experimental_ecma_transforms_base::remove_paren::remove_paren;

use super::BasicEvaluatedExpression;
use crate::visitors::JavascriptParser;

#[inline]
pub fn eval_source<'parser>(
  parser: &mut JavascriptParser<'parser>,
  source: String,
  error_title: String,
) -> Option<BasicEvaluatedExpression<'parser>> {
  let allocator = parser.ast.allocator;
  let source_in_allocator = allocator.alloc_str(&source);
  let result = parse_file_as_expr(
    allocator,
    source_in_allocator,
    Syntax::Es(EsSyntax::default()),
    EsVersion::EsNext,
    None,
  );
  match result {
    Err(err) => {
      let span = err.span();
      let mut error = Error::from_string(
        Some(source.clone()),
        span.start.saturating_sub(1) as usize,
        span.end.saturating_sub(1) as usize,
        format!("{error_title} warning"),
        format!("failed to parse {}", json!(source.as_str())),
      );
      error.severity = Severity::Warning;
      parser.add_warning(error.into());
      None
    }
    Ok(mut expr) => {
      remove_paren(&mut expr, allocator, None);
      BasicEvaluatedExpression::with_owned_expression(expr, |expr| {
        Some(parser.evaluate_expression(expr))
      })
    }
  }
}
