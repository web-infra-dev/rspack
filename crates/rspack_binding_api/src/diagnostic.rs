use napi::bindgen_prelude::*;
use rspack_error::{Diagnostic, Error as RspackError, Label, Severity};
use rspack_util::location::{
  try_line_column_length_to_location, try_line_column_length_to_offset_length,
};

#[napi(object)]
pub struct JsDiagnosticLocation {
  pub text: Option<String>,
  /// 1-based
  pub line: u32,
  /// 0-based in bytes
  pub column: u32,
  /// Length in bytes
  pub length: u32,
}

#[napi(object)]
pub struct JsDiagnostic {
  pub message: String,
  pub help: Option<String>,
  pub source_code: Option<String>,
  pub location: Option<JsDiagnosticLocation>,
  pub file: Option<String>,

  #[napi(ts_type = "\"error\" | \"warning\"")]
  pub severity: String,
  pub module_identifier: Option<String>,
}

#[napi(ts_return_type = "ExternalObject<'Diagnostic'>")]
pub fn format_diagnostic(diagnostic: JsDiagnostic) -> Result<External<Diagnostic>> {
  let JsDiagnostic {
    message,
    help,
    source_code,
    location,
    severity,
    module_identifier,
    file,
  } = diagnostic;
  let mut error = RspackError::error(message);
  error.severity = match severity.as_str() {
    "warning" => Severity::Warning,
    _ => Severity::Error,
  };
  if let Some(help) = help {
    error.help = Some(help);
  }
  let mut loc = None;
  if let Some(ref source_code) = source_code {
    let rope = ropey::Rope::from_str(source_code);
    if let Some(location) = location {
      loc = try_line_column_length_to_location(
        &rope,
        location.line as usize,
        location.column as usize,
        location.length as usize,
      );
      let (offset, length) = try_line_column_length_to_offset_length(
        &rope,
        location.line as usize,
        location.column as usize,
        location.length as usize,
      )
      .ok_or_else(|| {
        Error::new(
          Status::Unknown,
          "Format diagnostic failed: Invalid location. Did you pass the correct line, column and length?",
        )
      })?;
      let end_byte = offset.saturating_add(length);
      if end_byte > rope.len_bytes() {
        return Err(Error::new(
          Status::Unknown,
          "Format diagnostic failed: Invalid `length` in location.",
        ));
      }
      if !source_code.is_char_boundary(offset) || !source_code.is_char_boundary(end_byte) {
        return Err(Error::new(
          Status::Unknown,
          "Format diagnostic failed: Invalid char boundary. Did you pass the correct line, column and length?",
        ));
      }
      error.labels = Some(vec![Label {
        name: location.text,
        offset,
        len: length,
      }]);
    }
  }

  error.src = source_code;

  let mut diagnostic = Diagnostic::from(error);
  diagnostic.file = file.map(Into::into);
  diagnostic.loc = loc.map(|l| {
    rspack_core::DependencyLocation::Real(rspack_core::RealDependencyLocation {
      start: rspack_core::SourcePosition {
        line: l.sl as usize + 1,
        column: l.sc as usize,
      },
      end: Some(rspack_core::SourcePosition {
        line: l.el as usize + 1,
        column: l.ec as usize,
      }),
    })
  });
  diagnostic.module_identifier = module_identifier.map(Into::into);
  Ok(External::new(diagnostic))
}
