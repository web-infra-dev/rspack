mod graphical;

use miette::GraphicalTheme;

use self::graphical::GraphicalReportHandler;
use crate::{Result, diagnostic::Diagnostic, error::Error};

pub struct Renderer(GraphicalReportHandler);

impl Renderer {
  pub fn new(colored: bool) -> Self {
    let theme = if colored {
      GraphicalTheme::unicode()
    } else {
      GraphicalTheme::unicode_nocolor()
    };
    Self(
      GraphicalReportHandler::new()
        .with_theme(theme)
        .with_context_lines(2)
        .with_width(usize::MAX),
    )
  }

  pub fn render_error(&self, error: &Error) -> Result<String> {
    let mut buf = String::new();
    self.0.render_report(&mut buf, error)?;
    Ok(buf)
  }

  pub fn render_diagnostic(&self, diagnostic: &Diagnostic) -> Result<String> {
    self.render_error(&diagnostic.error)
  }
}
