mod graphical;

use miette::GraphicalTheme;

use self::graphical::GraphicalReportHandler;
use crate::{Result, error::Error};

pub struct Renderer {
  handler: GraphicalReportHandler,
  colored: bool,
}

impl Renderer {
  pub fn new(colored: bool) -> Self {
    let theme = if colored {
      GraphicalTheme::unicode()
    } else {
      GraphicalTheme::unicode_nocolor()
    };
    Self {
      handler: GraphicalReportHandler::new()
        .with_theme(theme)
        .with_context_lines(2)
        .with_width(usize::MAX),
      colored,
    }
  }

  pub fn render(&self, error: &Error) -> Result<String> {
    let cache = error.render_cache(self.colored);
    if let Some(rendered) = cache.get() {
      return Ok(rendered.clone());
    }

    let mut buf = String::new();
    self.handler.render_report(&mut buf, error)?;
    // Concurrent readers may populate the same cache; either report is equivalent.
    let _ = cache.set(buf.clone());
    Ok(buf)
  }
}
