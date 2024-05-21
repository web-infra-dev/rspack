use std::sync::Arc;

use rspack_core::EsVersion;
use rspack_error::{miette::Severity, TraceableError};
use swc_core::{
  common::{FileName, Spanned},
  ecma::parser::{parse_file_as_expr, EsConfig, Syntax},
};

use super::BasicEvaluatedExpression;
use crate::visitors::JavascriptParser;

pub fn eval_source(
  parser: &mut JavascriptParser,
  source: String,
  title: String,
) -> Option<BasicEvaluatedExpression> {
  let cm: Arc<swc_core::common::SourceMap> = Default::default();
  let fm = cm.new_source_file(FileName::Anon, source.clone());
  let result = parse_file_as_expr(
    &fm,
    Syntax::Es(EsConfig::default()),
    EsVersion::EsNext,
    None,
    &mut vec![],
  );
  match result {
    Err(err) => {
      let span = err.span();
      // Push the error to diagnostics
      parser.warning_diagnostics.push(Box::new(
        TraceableError::from_source_file(
          &fm,
          span.lo.0.saturating_sub(1) as usize,
          span.hi.0.saturating_sub(1) as usize,
          format!("{title} warning"),
          format!("failed to parse {:?}", source),
        )
        .with_severity(Severity::Warning),
      ));
      None
    }
    Ok(expr) => Some(parser.evaluate_expression(&expr)),
  }
}
