pub mod eval;
pub mod mangle_exports;
pub mod object_properties;

use rspack_core::ModuleType;
use rspack_error::Error;
use rspack_util::SpanExt;
use rustc_hash::FxHashSet as HashSet;
use swc_core::common::{SourceFile, Span, Spanned};

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
  module_type: &ModuleType,
) -> Error {
  let file_type = match module_type {
    ModuleType::JsAuto | ModuleType::JsDynamic | ModuleType::JsEsm => "JavaScript",
    _ => unreachable!("Only JavaScript module type is supported"),
  };
  Error::from_string(
    Some(fm.src.clone().into_string()),
    span.real_lo() as usize,
    span.real_hi() as usize,
    format!("{file_type} parse error"),
    message,
  )
}
