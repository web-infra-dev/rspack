use std::fmt::Display;

use rspack_error::{Error, Severity};
use rspack_util::SpanExt;
use serde_json::json;
use swc_experimental_ecma_ast::EsVersion;
use swc_experimental_ecma_parser::{EsSyntax, Syntax, parse_file_as_expr};
use swc_experimental_ecma_transforms_base::remove_paren::remove_paren;

use super::BasicEvaluatedExpression;
use crate::visitors::JavascriptParser;

#[inline]
pub fn eval_source<T: Display>(
  parser: &mut JavascriptParser,
  source: String,
  error_title: T,
) -> Option<BasicEvaluatedExpression> {
  let result = parse_file_as_expr(
    &source,
    Syntax::Es(EsSyntax::default()),
    EsVersion::EsNext,
    None,
  );
  match result {
    Err(err) => {
      // Push the error to diagnostics
      let mut error = Error::from_string(
        Some(source.clone()),
        err.span().real_lo() as usize,
        err.span().real_hi() as usize,
        format!("{error_title} warning"),
        format!("failed to parse {}", json!(source)),
      );
      error.severity = Severity::Warning;
      parser.add_warning(error.into());
      None
    }
    Ok(mut ret) => {
      remove_paren(ret.root, &mut ret.ast, None);
      BasicEvaluatedExpression::with_owned_expression(ret.root, |expr| {
        Some(parser.evaluate_expression(expr))
      })
    }
  }
}
