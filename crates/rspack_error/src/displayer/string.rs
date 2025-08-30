use std::io::Write;

use termcolor::{Buffer, WriteColor};

use super::{Display, renderer::Renderer};
use crate::diagnostic::Diagnostic;

#[derive(Default, Debug, Clone)]
pub struct StringDisplayer {
  colored: bool,
  sorted: bool,
}

impl StringDisplayer {
  pub fn new(colored: bool, sorted: bool) -> Self {
    Self { colored, sorted }
  }
}

impl Display for StringDisplayer {
  type Output = crate::Result<String>;

  fn emit_batch_diagnostic<'a>(
    &self,
    diagnostics: impl Iterator<Item = &'a Diagnostic>,
  ) -> Self::Output {
    let renderer = Renderer::new(self.colored);
    let mut diagnostic_strings = vec![];
    for d in diagnostics {
      diagnostic_strings.push(renderer.render(d)?);
    }
    if self.sorted {
      diagnostic_strings.sort_unstable();
    }
    if self.colored {
      let mut writer = Buffer::ansi();
      for s in diagnostic_strings {
        writer.write_all(s.as_bytes())?;
        writer.reset()?;
      }
      return Ok(String::from_utf8(writer.into_inner())?);
    }
    Ok(diagnostic_strings.join(""))
  }

  fn emit_diagnostic(&self, diagnostic: &Diagnostic) -> Self::Output {
    self.emit_batch_diagnostic(std::iter::once(diagnostic))
  }
}
