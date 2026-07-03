mod batch_error;
mod colors;
mod convert;
mod diagnosable;
mod diagnostic;
mod diagnostic_array;
mod displayer;
mod error;
mod macros;

pub use self::{
  batch_error::BatchErrors,
  colors::{cyan_str, dim_str, red_str, yellow_str},
  convert::{
    AnyhowResultToRspackResultExt, SerdeResultToRspackResultExt, ToStringResultToRspackResultExt,
    error_from_display, error_from_string, serde_error_with_detail,
  },
  diagnosable::Diagnosable,
  diagnostic::Diagnostic,
  diagnostic_array::{IntoTWithDiagnosticArray, TWithDiagnosticArray},
  displayer::{Display, Renderer, StdioDisplayer, StringDisplayer},
  error::{Error, ErrorData, Label, Severity},
};

pub type Result<T, E = Error> = std::result::Result<T, E>;
