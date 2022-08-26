use codespan_reporting::diagnostic::{Diagnostic, Label, Severity};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use dashmap::DashMap;
use std::io::Write;
use std::sync::Arc;
use termcolor::{Color, ColorSpec, WriteColor};

use crate::Diagnostic as RspackDiagnostic;

pub fn emit_batch_diagnostic(
  diagnostics: &Vec<RspackDiagnostic>,
  path_pos_map: Arc<DashMap<String, u32>>,
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

      // We now set up the writer and configuration, and then finally render the
      // diagnostic to standard error.

      let writer = StandardStream::stderr(ColorChoice::Always);
      let config = codespan_reporting::term::Config::default();

      term::emit(&mut writer.lock(), &config, &files, &diagnostic).unwrap();
    } else {
      let mut stderror = StandardStream::stderr(ColorChoice::Always);
      stderror.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
      writeln!(&mut stderror, "{}", diagnostic.message)?;
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
