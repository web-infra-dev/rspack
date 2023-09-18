use std::io::Write;
use std::path::Path;

use anyhow::Context;
use codespan_reporting::diagnostic::{Diagnostic, Label, Severity};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use codespan_reporting::term::{self, Config};
use sugar_path::SugarPath;
use termcolor::{Buffer, ColorSpec, StandardStreamLock, WriteColor};

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
    diagnostics: impl Iterator<Item = &RspackDiagnostic>,
  ) -> Self::Output;
  fn emit_diagnostic(&mut self, diagnostic: &RspackDiagnostic) -> Self::Output;
}

#[derive(Default)]
pub struct StdioDiagnosticDisplay;

impl DiagnosticDisplay for StdioDiagnosticDisplay {
  type Output = crate::Result<()>;

  fn emit_batch_diagnostic(
    &mut self,
    diagnostics: impl Iterator<Item = &RspackDiagnostic>,
  ) -> Self::Output {
    let writer = StandardStream::stderr(ColorChoice::Always);
    let mut lock_writer = writer.lock();
    emit_batch_diagnostic(diagnostics, &mut lock_writer)
  }

  fn emit_diagnostic(&mut self, diagnostic: &RspackDiagnostic) -> Self::Output {
    let writer = StandardStream::stderr(ColorChoice::Always);
    let mut lock_writer = writer.lock();
    let mut files = SimpleFiles::new();
    let pwd = std::env::current_dir()?;
    emit_diagnostic(diagnostic, &mut lock_writer, &pwd, &mut files)
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
    diagnostics: impl Iterator<Item = &RspackDiagnostic>,
  ) -> Self::Output {
    emit_batch_diagnostic(diagnostics, self)?;
    if self.sorted {
      self.diagnostic_vector.sort_unstable();
    }
    Ok(self.diagnostic_vector.drain(..).collect())
  }

  fn emit_diagnostic(&mut self, diagnostic: &RspackDiagnostic) -> Self::Output {
    let mut files = SimpleFiles::new();
    let pwd = std::env::current_dir()?;
    emit_diagnostic(diagnostic, self, &pwd, &mut files)?;
    self.flush_diagnostic();
    Ok(
      self
        .diagnostic_vector
        .pop()
        .context("diagnostic_vector should not empty after flush_diagnostic")?,
    )
  }
}

#[derive(Debug, Clone)]
pub struct ColoredStringDiagnosticDisplay;

impl DiagnosticDisplay for ColoredStringDiagnosticDisplay {
  type Output = crate::Result<String>;

  fn emit_batch_diagnostic(
    &mut self,
    diagnostics: impl Iterator<Item = &RspackDiagnostic>,
  ) -> Self::Output {
    let mut files = SimpleFiles::new();
    let pwd = std::env::current_dir()?;
    let mut buf = Buffer::ansi();
    for d in diagnostics {
      emit_diagnostic(d, &mut buf, &pwd, &mut files)?;
    }
    Ok(String::from_utf8_lossy(buf.as_slice()).to_string())
  }

  fn emit_diagnostic(&mut self, diagnostic: &RspackDiagnostic) -> Self::Output {
    let mut files = SimpleFiles::new();
    let pwd = std::env::current_dir()?;
    let mut buf = Buffer::ansi();
    emit_diagnostic(diagnostic, &mut buf, &pwd, &mut files)?;
    Ok(String::from_utf8_lossy(buf.as_slice()).to_string())
  }
}

fn emit_batch_diagnostic<T: Write + WriteColor + FlushDiagnostic>(
  diagnostics: impl Iterator<Item = &RspackDiagnostic>,
  writer: &mut T,
) -> crate::Result<()> {
  let mut files = SimpleFiles::new();
  let pwd = std::env::current_dir()?;

  for diagnostic in diagnostics {
    emit_diagnostic(diagnostic, writer, &pwd, &mut files)?;
    // `codespan_reporting` will not write the diagnostic message in a whole,
    // we need to insert some helper flag for sorting
    writer.flush_diagnostic();
  }
  Ok(())
}

fn emit_diagnostic<T: Write + WriteColor>(
  diagnostic: &RspackDiagnostic,
  writer: &mut T,
  pwd: impl AsRef<Path>,
  files: &mut SimpleFiles<String, String>,
) -> crate::Result<()> {
  let (labels, message) = match &diagnostic.source_info {
    Some(info) => {
      let file_path = Path::new(&info.path);
      let relative_path = file_path.relative(&pwd);
      let relative_path = relative_path.as_os_str().to_string_lossy().to_string();
      let file_id = files.add(relative_path, info.source.clone());
      (
        vec![Label::primary(file_id, diagnostic.start..diagnostic.end)
          .with_message(&diagnostic.message)],
        diagnostic.title.clone(),
      )
    }
    None => (vec![], diagnostic.message.clone()),
  };

  let diagnostic = Diagnostic::new(diagnostic.severity.into())
    .with_message(message)
    // Because we don't have error code now, and I don't think we have
    // enough energy to matain error code either in the future, so I use
    // this field to represent diagnostic kind, looks pretty neat.
    .with_code(diagnostic.kind.to_string())
    .with_notes(diagnostic.notes.clone())
    .with_labels(labels);

  let config = Config {
    before_label_lines: 4,
    after_label_lines: 4,
    ..Config::default()
  };

  term::emit(writer, &config, files, &diagnostic).expect("TODO:");
  // reset to original color after emitting a diagnostic, this avoids interference stdio of other procedure.
  writer.reset().map_err(|e| e.into())
}

impl From<crate::Severity> for Severity {
  fn from(severity: crate::Severity) -> Self {
    match severity {
      crate::Severity::Error => Self::Error,
      crate::Severity::Warn => Self::Warning,
    }
  }
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
    diagnostics: impl Iterator<Item = &RspackDiagnostic>,
  ) -> Self::Output {
    match self {
      Self::Colored(d) => d.emit_batch_diagnostic(diagnostics),
      Self::Plain(d) => d.emit_batch_diagnostic(diagnostics),
    }
  }

  fn emit_diagnostic(&mut self, diagnostic: &RspackDiagnostic) -> Self::Output {
    match self {
      Self::Colored(d) => d.emit_diagnostic(diagnostic),
      Self::Plain(d) => d.emit_diagnostic(diagnostic),
    }
  }
}
