pub mod eval;
mod get_prop_from_obj;
pub mod mangle_exports;
pub(crate) mod queue;

use rspack_core::{ErrorSpan, ModuleType};
use rspack_error::{DiagnosticKind, TraceableError};
use rustc_hash::FxHashSet as HashSet;
use swc_core::common::{SourceFile, Span, Spanned};

pub use self::get_prop_from_obj::*;

#[derive(PartialEq, Eq, Hash)]
pub struct EcmaError(String, Span);
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
  module_type: &ModuleType,
) -> TraceableError {
  let (file_type, diagnostic_kind) = match module_type {
    ModuleType::Js | ModuleType::JsDynamic | ModuleType::JsEsm => {
      ("JavaScript", DiagnosticKind::JavaScript)
    }
    _ => unreachable!("Only JavaScript module type is supported"),
  };
  let span: ErrorSpan = span.into();
  rspack_error::TraceableError::from_source_file(
    fm,
    span.start as usize,
    span.end as usize,
    format!("{file_type} parsing error"),
    message,
  )
  .with_kind(diagnostic_kind)
}

pub fn is_diff_mode() -> bool {
  let is_diff_mode = std::env::var("RSPACK_DIFF").ok().unwrap_or_default();
  is_diff_mode == "true"
}
