use std::io::Write;

use anyhow::Context;
use miette::IntoDiagnostic;
use termcolor::{Buffer, ColorSpec, StandardStreamLock, WriteColor};
use termcolor::{ColorChoice, StandardStream};

use crate::Diagnostic;

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
    diagnostics: impl Iterator<Item = &Diagnostic>,
  ) -> Self::Output;
  fn emit_diagnostic(&mut self, diagnostic: &Diagnostic) -> Self::Output;
}

#[derive(Default)]
pub struct StdioDiagnosticDisplay;

impl DiagnosticDisplay for StdioDiagnosticDisplay {
  type Output = crate::Result<()>;

  fn emit_batch_diagnostic(
    &mut self,
    diagnostics: impl Iterator<Item = &Diagnostic>,
  ) -> Self::Output {
    let writer = StandardStream::stderr(ColorChoice::Always);
    let mut lock_writer = writer.lock();
    emit_batch_diagnostic(diagnostics, &mut lock_writer)
  }

  fn emit_diagnostic(&mut self, diagnostic: &Diagnostic) -> Self::Output {
    let writer = StandardStream::stderr(ColorChoice::Always);
    let mut lock_writer = writer.lock();
    emit_diagnostic(diagnostic, &mut lock_writer)
  }
}

#[derive(Default, Debug, Clone)]
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
    diagnostics: impl Iterator<Item = &Diagnostic>,
  ) -> Self::Output {
    emit_batch_diagnostic(diagnostics, self)?;
    if self.sorted {
      self.diagnostic_vector.sort_unstable();
    }
    Ok(self.diagnostic_vector.drain(..).collect())
  }

  fn emit_diagnostic(&mut self, diagnostic: &Diagnostic) -> Self::Output {
    emit_diagnostic(diagnostic, self)?;
    self.flush_diagnostic();
    self
      .diagnostic_vector
      .pop()
      .context("diagnostic_vector should not empty after flush_diagnostic")
      .map_err(|e| miette::miette!(e.to_string()))
  }
}

#[derive(Debug, Clone)]
pub struct ColoredStringDiagnosticDisplay;

impl DiagnosticDisplay for ColoredStringDiagnosticDisplay {
  type Output = crate::Result<String>;

  fn emit_batch_diagnostic(
    &mut self,
    diagnostics: impl Iterator<Item = &Diagnostic>,
  ) -> Self::Output {
    let mut buf = Buffer::ansi();
    for d in diagnostics {
      emit_diagnostic(d, &mut buf)?;
    }
    Ok(String::from_utf8_lossy(buf.as_slice()).to_string())
  }

  fn emit_diagnostic(&mut self, diagnostic: &Diagnostic) -> Self::Output {
    let mut buf = Buffer::ansi();
    emit_diagnostic(diagnostic, &mut buf)?;
    Ok(String::from_utf8_lossy(buf.as_slice()).to_string())
  }
}

fn emit_batch_diagnostic<T: Write + WriteColor + FlushDiagnostic>(
  diagnostics: impl Iterator<Item = &Diagnostic>,
  writer: &mut T,
) -> crate::Result<()> {
  for diagnostic in diagnostics {
    emit_diagnostic(diagnostic, writer)?;
    // `codespan_reporting` will not write the diagnostic message in a whole,
    // we need to insert some helper flag for sorting
    writer.flush_diagnostic();
  }
  Ok(())
}

fn emit_diagnostic<T: Write + WriteColor>(
  diagnostic: &Diagnostic,
  writer: &mut T,
) -> crate::Result<()> {
  let buf = diagnostic.render_report(writer.supports_color())?;
  writer.write_all(buf.as_bytes()).into_diagnostic()?;
  // reset to original color after emitting a diagnostic, this avoids interference stdio of other procedure.
  writer.reset().into_diagnostic()?;
  Ok(())
}

#[derive(Debug, Clone)]
pub enum DiagnosticDisplayer {
  Colored(ColoredStringDiagnosticDisplay),
  Plain(StringDiagnosticDisplay),
}

impl DiagnosticDisplayer {
  pub fn new(colored: bool) -> Self {
    if colored {
      Self::Colored(ColoredStringDiagnosticDisplay)
    } else {
      Self::Plain(StringDiagnosticDisplay::default())
    }
  }
}

impl DiagnosticDisplay for DiagnosticDisplayer {
  type Output = crate::Result<String>;

  fn emit_batch_diagnostic(
    &mut self,
    diagnostics: impl Iterator<Item = &Diagnostic>,
  ) -> Self::Output {
    match self {
      Self::Colored(d) => d.emit_batch_diagnostic(diagnostics),
      Self::Plain(d) => d.emit_batch_diagnostic(diagnostics),
    }
  }

  fn emit_diagnostic(&mut self, diagnostic: &Diagnostic) -> Self::Output {
    match self {
      Self::Colored(d) => d.emit_diagnostic(diagnostic),
      Self::Plain(d) => d.emit_diagnostic(diagnostic),
    }
  }
}
