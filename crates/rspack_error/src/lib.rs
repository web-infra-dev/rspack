mod batch_error;
mod convert;
mod diagnosable;
mod diagnostic;
mod diagnostic_array;
mod displayer;
mod error;
mod macros;

pub use self::{
  batch_error::BatchErrors,
  convert::{
    AnyhowResultToRspackResultExt, SerdeResultToRspackResultExt, ToStringResultToRspackResultExt,
  },
  diagnosable::Diagnosable,
  diagnostic::Diagnostic,
  diagnostic_array::{IntoTWithDiagnosticArray, TWithDiagnosticArray},
  displayer::{Display, Renderer, StdioDisplayer, StringDisplayer},
  error::{Error, ErrorData, Label, Severity},
};

pub type Result<T, E = Error> = std::result::Result<T, E>;
