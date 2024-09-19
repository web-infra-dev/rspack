use napi::bindgen_prelude::*;
use rspack_error::{
  miette::{self, LabeledSpan, MietteDiagnostic, Severity},
  Diagnostic,
};
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
  let mut d = MietteDiagnostic::new(message).with_severity(match severity.as_str() {
    "warning" => Severity::Warning,
    _ => Severity::Error,
  });
  if let Some(help) = help {
    d = d.with_help(help);
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
      d = d.with_label(LabeledSpan::new(location.text, offset, length));
    }
  }

  let mut error = miette::Error::new(d);
  if let Some(source_code) = source_code {
    error = error.with_source_code(source_code);
  }
  Ok(External::new(
    Diagnostic::from(error)
      .with_file(file.map(Into::into))
      .with_loc(loc.map(|l| l.to_string()))
      .with_module_identifier(module_identifier.map(Into::into)),
  ))
}
