// TODO: remove  EcmaError, EcmaErrorsDeduped in rspack_plugin_javascript

use rspack_error::{DiagnosticKind, TraceableError};
use rustc_hash::FxHashSet as HashSet;
use swc_core::common::{SourceFile, Span};

/// Using `u32` instead of `usize` to reduce memory usage,
/// `u32` is 4 bytes on 64bit machine, comparing to `usize` which is 8 bytes.
/// ## Warning
/// [ErrorSpan] start from zero, and `Span` of `swc` start from one. see https://swc-css.netlify.app/?code=eJzLzC3ILypRSFRIK8rPVVAvSS0u0csqVgcAZaoIKg
#[cacheable]
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Default, PartialOrd, Ord)]
pub struct ErrorSpan {
  pub start: u32,
  pub end: u32,
}

impl ErrorSpan {
  pub fn new(start: u32, end: u32) -> Self {
    Self { start, end }
  }
}

impl From<Span> for ErrorSpan {
  fn from(span: Span) -> Self {
    Self {
      start: span.lo.0.saturating_sub(1),
      end: span.hi.0.saturating_sub(1),
    }
  }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct EcmaError(String, Span);

#[derive(Debug)]
pub struct EcmaErrorsDeduped(Vec<EcmaError>);

impl IntoIterator for EcmaErrorsDeduped {
  type Item = EcmaError;
  type IntoIter = std::vec::IntoIter<Self::Item>;
  fn into_iter(self) -> Self::IntoIter {
    self.0.into_iter()
  }
}

impl From<Vec<swc_core::ecma::parser::error::Error>> for EcmaErrorsDeduped {
  fn from(value: Vec<swc_core::ecma::parser::error::Error>) -> Self {
    Self(
      value
        .into_iter()
        .map(|v| EcmaError(v.kind().msg().to_string(), v.span()))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>(),
    )
  }
}

/// Dedup ecma errors against [swc_core::ecma::parser::error::Error]
/// Returns a wrapper of an iterator that contains deduped errors.
impl DedupEcmaErrors for Vec<swc_core::ecma::parser::error::Error> {
  fn dedup_ecma_errors(self) -> EcmaErrorsDeduped {
    EcmaErrorsDeduped::from(self)
  }
}

pub trait DedupEcmaErrors {
  fn dedup_ecma_errors(self) -> EcmaErrorsDeduped;
}

pub fn ecma_parse_error_deduped_to_rspack_error(
  EcmaError(message, span): EcmaError,
  fm: &SourceFile,
) -> TraceableError {
  let span: ErrorSpan = span.into();
  rspack_error::TraceableError::from_source_file(
    fm,
    span.start,
    span.end,
    "JavaScript parsing error".into(),
    message,
  )
  .with_kind(DiagnosticKind::JavaScript)
}
