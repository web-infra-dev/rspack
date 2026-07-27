use std::{
  fmt::Debug,
  sync::{Arc, mpsc},
};

use rspack_error::{BatchErrors, Error, Label, Severity, error};
use rspack_util::SpanExt;
use rustc_hash::FxHashSet as HashSet;
use swc_core::common::{
  SourceMap, Span, Spanned,
  errors::{Diagnostic as SwcDiagnostic, DiagnosticId, Emitter, HANDLER, Handler, Level},
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
    "JavaScript parse error".into(),
    message,
  )
}

/// Converts structured SWC diagnostics into a Rspack error chain.
///
/// SWC spans use absolute byte positions within a [`SourceMap`], while Rspack
/// errors expect byte offsets relative to the source file. Resolve the spans
/// here instead of rendering them to text so Rspack can render the diagnostics
/// together with its own error context.
pub(crate) fn swc_diagnostics_to_rspack_error(
  diagnostics: &[SwcDiagnostic],
  source_map: &SourceMap,
) -> Option<Error> {
  let mut errors = diagnostics
    .iter()
    .rev()
    .map(|diagnostic| swc_diagnostic_to_rspack_error(diagnostic, source_map));
  let mut error = errors.next()?;

  for mut outer in errors {
    outer.source_error = Some(Box::new(error));
    error = outer;
  }

  if diagnostics.iter().any(SwcDiagnostic::is_error) {
    error.severity = Severity::Error;
  }

  Some(error)
}

fn swc_diagnostic_to_rspack_error(diagnostic: &SwcDiagnostic, source_map: &SourceMap) -> Error {
  let mut message = diagnostic.message();
  let code = diagnostic.code.as_ref().map(|code| match code {
    DiagnosticId::Error(code) | DiagnosticId::Lint(code) => code,
  });
  if let Some(code) = code {
    message.insert_str(0, ": ");
    message.insert_str(0, code);
  }

  let mut error = match diagnostic.level {
    Level::Warning | Level::Note | Level::Help => Error::warning(message),
    Level::Bug
    | Level::Fatal
    | Level::PhaseFatal
    | Level::Error
    | Level::FailureNote
    | Level::Cancelled => Error::error(message),
  };

  error.code = code.cloned();

  if let Some(primary_span) = diagnostic.span.primary_span()
    && let Ok(primary) = source_map.try_lookup_byte_offset(primary_span.lo())
  {
    let source_file = primary.sf;
    let labels = diagnostic
      .span
      .span_labels()
      .into_iter()
      .filter_map(|label| {
        if label.span.lo() < source_file.start_pos || label.span.hi() > source_file.end_pos {
          return None;
        }

        Some(Label {
          name: label.label,
          offset: (label.span.lo() - source_file.start_pos).0 as usize,
          len: (label.span.hi() - label.span.lo()).0 as usize,
        })
      })
      .collect::<Vec<_>>();

    error.src = Some(source_file.src.to_string());
    if !labels.is_empty() {
      error.labels = Some(labels);
    }
  }

  let help = diagnostic
    .children
    .iter()
    .filter(|child| child.level == Level::Help)
    .map(|child| child.message())
    .collect::<Vec<_>>()
    .join("\n");
  if !help.is_empty() {
    error.help = Some(help);
  }

  error
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
