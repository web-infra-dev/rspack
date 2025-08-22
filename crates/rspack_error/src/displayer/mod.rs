mod renderer;
mod stdio;
mod string;

pub use self::{renderer::Renderer, stdio::StdioDisplayer, string::StringDisplayer};
use crate::diagnostic::Diagnostic;

pub trait Display {
  type Output;
  fn emit_batch_diagnostic<'a>(
    &self,
    diagnostics: impl Iterator<Item = &'a Diagnostic>,
  ) -> Self::Output;
  fn emit_diagnostic(&self, diagnostic: &Diagnostic) -> Self::Output;
}
