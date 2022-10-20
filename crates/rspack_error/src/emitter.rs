use codespan_reporting::diagnostic::{Diagnostic, Label, Severity};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use dashmap::DashMap;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use sugar_path::PathSugar;
use termcolor::{Color, ColorSpec, StandardStreamLock, WriteColor};

use crate::Diagnostic as RspackDiagnostic;

pub trait FlushDiagnostic {
  fn flush_diagnostic(&mut self) {}
}

impl FlushDiagnostic for StringDiagnosticDisplay {
  fn flush_diagnostic(&mut self) {
    self
      .diagnostic_vector
      .push(std::mem::take(&mut self.string_buffer).join(""));
  }
}

impl FlushDiagnostic for StandardStreamLock<'_> {}
pub trait DiagnosticDisplay {
  type Output;
  fn emit_batch_diagnostic(
    &mut self,
    diagnostics: &[RspackDiagnostic],
    path_pos_map: Arc<DashMap<String, u32>>,
  ) -> Self::Output;
}

#[derive(Default)]
pub struct StdioDiagnosticDisplay {}

impl DiagnosticDisplay for StdioDiagnosticDisplay {
  type Output = crate::Result<()>;

  fn emit_batch_diagnostic(
    &mut self,
    diagnostics: &[RspackDiagnostic],
    path_pos_map: Arc<DashMap<String, u32>>,
  ) -> Self::Output {
    let writer = StandardStream::stderr(ColorChoice::Always);
    let mut lock_writer = writer.lock();
    emit_batch_diagnostic(diagnostics, path_pos_map, &mut lock_writer)
  }
}

#[derive(Default)]
pub struct StringDiagnosticDisplay {
  string_buffer: Vec<String>,
  sorted: bool,
  diagnostic_vector: Vec<String>,
}

impl StringDiagnosticDisplay {
  pub fn with_sorted(self, sorted: bool) -> Self {
    Self { sorted, ..self }
  }
}

impl Write for StringDiagnosticDisplay {
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    let len = buf.len();
    self
      .string_buffer
      .push(String::from_utf8_lossy(buf).to_string());
    Ok(len)
  }

  fn flush(&mut self) -> std::io::Result<()> {
    Ok(())
  }
}
impl WriteColor for StringDiagnosticDisplay {
  fn supports_color(&self) -> bool {
    false
  }

  fn set_color(&mut self, _: &ColorSpec) -> std::io::Result<()> {
    Ok(())
  }

  fn reset(&mut self) -> std::io::Result<()> {
    Ok(())
  }
}

impl DiagnosticDisplay for StringDiagnosticDisplay {
  type Output = crate::Result<String>;

  fn emit_batch_diagnostic(
    &mut self,
    diagnostics: &[RspackDiagnostic],
    path_pos_map: Arc<DashMap<String, u32>>,
  ) -> Self::Output {
    emit_batch_diagnostic(diagnostics, path_pos_map, self)?;
    if self.sorted {
      self.diagnostic_vector.sort();
    }
    Ok(self.diagnostic_vector.join(""))
  }
}
fn emit_batch_diagnostic<T: Write + WriteColor + FlushDiagnostic>(
  diagnostics: &[RspackDiagnostic],
  path_pos_map: Arc<DashMap<String, u32>>,
  writer: &mut T,
) -> crate::Result<()> {
  let mut files = SimpleFiles::new();
  let pwd = std::env::current_dir()?;

  dbg!(&diagnostics);
  for diagnostic in diagnostics {
    if let Some(info) = &diagnostic.source_info {
      // Since `Span` of `swc` started with 1 and span of diagnostic started with 0
      // So we need to subtract 1 to `start_relative_sourcemap`;
      let start_relative_sourcemap = path_pos_map
        .get(&info.path)
        .map(|v| *v)
        .unwrap_or(0)
        .saturating_sub(1) as usize;
      let start = diagnostic.start - start_relative_sourcemap;
      let end = diagnostic.end - start_relative_sourcemap;
      let file_path = Path::new(&info.path);
      let relative_path = file_path.relative(&pwd);
      let relative_path = relative_path.as_os_str().to_string_lossy().to_string();
      let file_id = files.add(relative_path, info.source.clone());
      let diagnostic = Diagnostic::new(diagnostic.severity.into())
        .with_message(&diagnostic.title)
        // Because we don't have error code now, and I don't think we have
        // enough energy to matain error code either in the future, so I use
        // this field to represent diagnostic kind, looks pretty neat.
        .with_code(diagnostic.kind.to_string())
        .with_labels(vec![
          Label::primary(file_id, start..end).with_message(&diagnostic.message)
        ]);

      let config = codespan_reporting::term::Config::default();

      term::emit(writer, &config, &files, &diagnostic).unwrap();
      // `codespan_reporting` will not write the diagnostic message in a whole,
      // we need to insert some helper flag for sorting
      writer.flush_diagnostic();
    } else {
      writer.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
      writeln!(writer, "{}", diagnostic.message)?;
      writer.flush_diagnostic();
    }
  }
  Ok(())
}

impl From<crate::Severity> for Severity {
  fn from(severity: crate::Severity) -> Self {
    match severity {
      crate::Severity::Error => Self::Error,
      crate::Severity::Warn => Self::Warning,
    }
  }
}
