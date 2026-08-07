mod graphical;

use std::{borrow::Cow, fmt};

use miette::{
  Diagnostic as MietteDiagnostic, GraphicalTheme, LabeledSpan, MietteError, SourceCode, SourceSpan,
  SpanContents,
};

use self::graphical::GraphicalReportHandler;
use crate::{Result, error::Error};

struct RenderSource<'a>(Cow<'a, str>);

impl SourceCode for RenderSource<'_> {
  fn read_span<'a>(
    &'a self,
    span: &SourceSpan,
    context_lines_before: usize,
    context_lines_after: usize,
  ) -> std::result::Result<Box<dyn SpanContents<'a> + 'a>, MietteError> {
    self
      .0
      .read_span(span, context_lines_before, context_lines_after)
  }
}

// `miette::Diagnostic::source_code` can only borrow source data stored by the diagnostic. This
// wrapper owns a flattened source for exactly one render, allowing composite sources to provide
// snippets without caching a second full source copy on `Error`.
struct RenderDiagnostic<'a> {
  error: &'a Error,
  source: Option<RenderSource<'a>>,
  source_error: Option<Box<RenderDiagnostic<'a>>>,
}

impl<'a> RenderDiagnostic<'a> {
  fn new(error: &'a Error) -> Self {
    Self {
      error,
      source: error
        .src
        .as_ref()
        .map(|source| RenderSource(source.source().into_string_lossy())),
      source_error: error.source_error.as_deref().map(Self::new).map(Box::new),
    }
  }
}

impl fmt::Debug for RenderDiagnostic<'_> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    fmt::Debug::fmt(self.error, f)
  }
}

impl fmt::Display for RenderDiagnostic<'_> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    fmt::Display::fmt(self.error, f)
  }
}

impl std::error::Error for RenderDiagnostic<'_> {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    std::error::Error::source(self.error)
  }
}

impl MietteDiagnostic for RenderDiagnostic<'_> {
  fn code(&self) -> Option<Box<dyn fmt::Display + '_>> {
    MietteDiagnostic::code(self.error)
  }

  fn severity(&self) -> Option<miette::Severity> {
    MietteDiagnostic::severity(self.error)
  }

  fn help(&self) -> Option<Box<dyn fmt::Display + '_>> {
    MietteDiagnostic::help(self.error)
  }

  fn url(&self) -> Option<Box<dyn fmt::Display + '_>> {
    MietteDiagnostic::url(self.error)
  }

  fn source_code(&self) -> Option<&dyn SourceCode> {
    self.source.as_ref().map(|source| source as &dyn SourceCode)
  }

  fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
    MietteDiagnostic::labels(self.error)
  }

  fn related(&self) -> Option<Box<dyn Iterator<Item = &dyn MietteDiagnostic> + '_>> {
    MietteDiagnostic::related(self.error)
  }

  fn diagnostic_source(&self) -> Option<&dyn MietteDiagnostic> {
    self
      .source_error
      .as_deref()
      .map(|error| error as &dyn MietteDiagnostic)
  }
}

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

  pub fn render(&self, error: &Error) -> Result<String> {
    let mut buf = String::new();
    let diagnostic = RenderDiagnostic::new(error);
    self.0.render_report(&mut buf, &diagnostic)?;
    Ok(buf)
  }
}
