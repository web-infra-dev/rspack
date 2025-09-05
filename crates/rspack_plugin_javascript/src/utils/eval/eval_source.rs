use std::{fmt::Display, sync::Arc};

use rspack_error::{Error, Severity};
use rspack_util::SpanExt;
use serde_json::json;
use swc_core::{
  common::{FileName, Spanned},
  ecma::{
    ast::EsVersion,
    parser::{EsSyntax, Syntax, parse_file_as_expr},
    transforms::base::fixer::paren_remover,
    visit::VisitMutWith,
  },
};

use super::BasicEvaluatedExpression;
use crate::visitors::JavascriptParser;

#[inline]
pub fn eval_source<T: Display>(
  parser: &mut JavascriptParser,
  source: String,
  error_title: T,
) -> Option<BasicEvaluatedExpression<'static>> {
  let cm: Arc<swc_core::common::SourceMap> = Default::default();
  let fm = cm.new_source_file(Arc::new(FileName::Anon), source);
  let result = parse_file_as_expr(
    &fm,
    Syntax::Es(EsSyntax::default()),
    EsVersion::EsNext,
    None,
    &mut vec![],
  );
  match result {
    Err(err) => {
      // Push the error to diagnostics
      let mut error = Error::from_string(
        Some(fm.src.clone().into_string()),
        err.span().real_lo() as usize,
        err.span().real_hi() as usize,
        format!("{error_title} warning"),
        format!("failed to parse {}", json!(fm.src.as_str())),
      );
      error.severity = Severity::Warning;
      parser.add_warning(error.into());
      None
    }
    Ok(mut expr) => {
      expr.visit_mut_with(&mut paren_remover(None));
      BasicEvaluatedExpression::with_owned_expression(*expr, |expr| {
        Some(parser.evaluate_expression(expr))
      })
    }
  }
}
