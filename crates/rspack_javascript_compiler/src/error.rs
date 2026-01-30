use std::{
  fmt::Debug,
  sync::{Arc, mpsc},
};

use rspack_error::{BatchErrors, Error, error};
use rspack_util::SpanExt;
use rustc_hash::FxHashSet as HashSet;
use swc_core::common::{
  SourceMap, Span, Spanned,
  errors::{Emitter, HANDLER, Handler},
};

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
  source: String,
) -> Error {
  Error::from_string(
    Some(source),
    span.real_lo() as usize,
    span.real_hi() as usize,
    "JavaScript parse error",
    message,
  )
}

// keep this private to make sure with_rspack_error_handler is safety
struct RspackErrorEmitter {
  tx: mpsc::Sender<Error>,
  source_map: Arc<SourceMap>,
  title: String,
}

impl Emitter for RspackErrorEmitter {
  fn emit(&mut self, db: &mut swc_core::common::errors::DiagnosticBuilder<'_>) {
    let source_file_and_byte_pos = db
      .span
      .primary_span()
      .map(|s| self.source_map.lookup_byte_offset(s.lo()));
    if let Some(source_file_and_byte_pos) = source_file_and_byte_pos {
      self
        .tx
        .send(Error::from_string(
          Some(source_file_and_byte_pos.sf.src.clone().into_string()),
          source_file_and_byte_pos.pos.0 as usize,
          source_file_and_byte_pos.pos.0 as usize,
          self.title.clone(),
          db.message(),
        ))
        .expect("Sender should drop after emit called");
    } else {
      self
        .tx
        .send(error!(db.message()))
        .expect("Sender should drop after emit called");
    }
  }
}

/// Executes a closure with an error handler and returns the result or a BatchErrors if errors occurred.
///
/// This function sets up an error handler with a custom emitter that sends errors to a channel. It then
/// executes the provided closure with a reference to the handler. If the handler has errors after the
/// closure execution, it collects the errors from the channel and returns them as a BatchErrors. If no
/// errors occurred, it returns the result of the closure.
///
/// # Parameters
///
/// - `title`: The title of the error handler.
/// - `kind`: The kind of diagnostic to use for errors.
/// - `cm`: The source map to use for error reporting.
/// - `op`: The closure to execute with the error handler.
///
/// # Returns
///
/// A result containing either the return value of the closure or a BatchErrors if errors occurred.
pub fn with_rspack_error_handler<F, Ret>(
  title: String,
  cm: Arc<SourceMap>,
  op: F,
) -> std::result::Result<Ret, BatchErrors>
where
  F: FnOnce(&Handler) -> std::result::Result<Ret, BatchErrors>,
{
  let (tx, rx) = mpsc::channel();
  let emitter = RspackErrorEmitter {
    title,
    source_map: cm,
    tx,
  };
  let handler = Handler::with_emitter(true, false, Box::new(emitter));

  let ret = HANDLER.set(&handler, || op(&handler));

  if handler.has_errors() {
    drop(handler);
    Err(BatchErrors(rx.into_iter().collect()))
  } else {
    ret
  }
}
