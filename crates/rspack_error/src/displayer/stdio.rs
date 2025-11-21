use std::io::Write;

use termcolor::{ColorChoice, StandardStream, WriteColor};

use super::{Display, renderer::Renderer};
use crate::{Result, diagnostic::Diagnostic};

#[derive(Default)]
pub struct StdioDisplayer;

impl Display for StdioDisplayer {
  type Output = Result<()>;

  fn emit_batch_diagnostic<'a>(
    &self,
    diagnostics: impl Iterator<Item = &'a Diagnostic>,
  ) -> Self::Output {
    let writer = StandardStream::stderr(ColorChoice::Always);
    let mut lock_writer = writer.lock();
    let renderer = Renderer::new(lock_writer.supports_color());
    for diagnostic in diagnostics {
      let buf = renderer.render(diagnostic)?;
      lock_writer.write_all(buf.as_bytes())?;
      // reset to original color after emitting a diagnostic, this avoids interference stdio of other procedure.
      lock_writer.reset()?;
    }
    Ok(())
  }

  fn emit_diagnostic(&self, diagnostic: &Diagnostic) -> Self::Output {
    self.emit_batch_diagnostic(vec![diagnostic].into_iter())
  }
}
