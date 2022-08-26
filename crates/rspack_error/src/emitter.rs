use codespan_reporting::diagnostic::{Diagnostic, Label, Severity};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use dashmap::DashMap;
use std::io::Write;
use std::sync::Arc;
use termcolor::{Color, ColorSpec, WriteColor};

use crate::Diagnostic as RspackDiagnostic;
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
  inner: String,
}

impl Write for StringDiagnosticDisplay {
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    let len = buf.len();
    self.inner.push_str(&String::from_utf8_lossy(buf));
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
    let ret = std::mem::take(&mut self.inner);
    Ok(ret)
  }
}
fn emit_batch_diagnostic<T: Write + WriteColor>(
  diagnostics: &[RspackDiagnostic],
  path_pos_map: Arc<DashMap<String, u32>>,
  writer: &mut T,
) -> crate::Result<()> {
  let mut files = SimpleFiles::new();
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
      let file_id = files.add(info.path.clone(), info.source.clone());
      let diagnostic = Diagnostic::new(diagnostic.severity.into())
        .with_message(&diagnostic.title)
        .with_labels(vec![
          Label::primary(file_id, start..end).with_message(&diagnostic.message)
        ]);

      let config = codespan_reporting::term::Config::default();

      term::emit(writer, &config, &files, &diagnostic).unwrap();
    } else {
      writer.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
      writeln!(writer, "{}", diagnostic.message)?;
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
