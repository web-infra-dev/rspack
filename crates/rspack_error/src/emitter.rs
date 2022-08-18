use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use std::io::Write;
use termcolor::{Color, ColorSpec, WriteColor};

use crate::Diagnostic as RspackDiagnostic;

pub fn emit_batch_diagnostic(diagnostics: &Vec<RspackDiagnostic>) -> crate::Result<()> {
  let mut files = SimpleFiles::new();
  for diagnostic in diagnostics {
    if let Some(info) = &diagnostic.source_info {
      let start = diagnostic.start;
      let end = diagnostic.end;
      let file_id = files.add(info.path.clone(), info.source.clone());
      let diagnostic = Diagnostic::error()
        .with_message(&diagnostic.message)
        .with_labels(vec![
          Label::primary(file_id, start..end).with_message("expected `String`, found `Nat`")
        ]);

      // We now set up the writer and configuration, and then finally render the
      // diagnostic to standard error.

      let writer = StandardStream::stderr(ColorChoice::Always);
      let config = codespan_reporting::term::Config::default();

      term::emit(&mut writer.lock(), &config, &files, &diagnostic).unwrap();
    } else {
      let mut stderror = StandardStream::stderr(ColorChoice::Always);
      stderror.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
      writeln!(&mut stderror, "testjiofoea",)?;
      println!("{}", diagnostic.message);
    }
  }
  Ok(())
}
