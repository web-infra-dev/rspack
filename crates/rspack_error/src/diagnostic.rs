use std::{
  borrow::Cow,
  fmt,
  ops::Deref,
  sync::{Arc, LazyLock},
};

use cow_utils::CowUtils;
use miette::{GraphicalTheme, IntoDiagnostic, MietteDiagnostic};
use rspack_cacheable::{cacheable, with::Unsupported};
use rspack_collections::Identifier;
use rspack_location::DependencyLocation;
use rspack_paths::{Utf8Path, Utf8PathBuf};

use crate::{Error, graphical::GraphicalReportHandler};

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, Hash)]
pub enum RspackSeverity {
  #[default]
  Error,
  Warn,
}

pub type Severity = RspackSeverity;

impl From<RspackSeverity> for miette::Severity {
  fn from(value: RspackSeverity) -> Self {
    match value {
      RspackSeverity::Error => miette::Severity::Error,
      RspackSeverity::Warn => miette::Severity::Warning,
    }
  }
}

impl From<miette::Severity> for RspackSeverity {
  fn from(value: miette::Severity) -> Self {
    match value {
      miette::Severity::Error => RspackSeverity::Error,
      miette::Severity::Warning => RspackSeverity::Warn,
      miette::Severity::Advice => unimplemented!("Not supported miette severity"),
    }
  }
}

impl From<&str> for RspackSeverity {
  fn from(value: &str) -> Self {
    let s = value.cow_to_ascii_lowercase();
    match s.as_ref() {
      "warning" => RspackSeverity::Warn,
      _ => RspackSeverity::Error,
    }
  }
}

impl fmt::Display for RspackSeverity {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{}",
      match self {
        RspackSeverity::Error => "error",
        RspackSeverity::Warn => "warning",
      }
    )
  }
}

#[cacheable]
#[derive(Debug, Clone, Copy)]
pub struct SourcePosition {
  pub line: usize,
  pub column: usize,
}

#[cacheable(with=Unsupported)]
#[derive(Debug, Clone)]
pub struct Diagnostic {
  inner: Arc<miette::Error>,

  // The following fields are only used to restore Diagnostic for Rspack.
  // If the current Diagnostic originates from Rust, these fields will be None.
  details: Option<String>,
  module_identifier: Option<Identifier>,
  loc: Option<DependencyLocation>,
  file: Option<Utf8PathBuf>,
  hide_stack: Option<bool>,
  chunk: Option<u32>,
  stack: Option<String>,
}

impl From<Box<dyn miette::Diagnostic + Send + Sync>> for Diagnostic {
  fn from(value: Box<dyn miette::Diagnostic + Send + Sync>) -> Self {
    Diagnostic::from(miette::Error::new_boxed(value))
  }
}

impl From<Arc<miette::Error>> for Diagnostic {
  fn from(value: Arc<miette::Error>) -> Self {
    Self {
      inner: value,
      details: None,
      module_identifier: None,
      loc: None,
      file: None,
      hide_stack: None,
      chunk: None,
      stack: None,
    }
  }
}

impl From<miette::Error> for Diagnostic {
  fn from(value: miette::Error) -> Self {
    Self {
      inner: Arc::new(value),
      details: None,
      module_identifier: None,
      loc: None,
      file: None,
      hide_stack: None,
      chunk: None,
      stack: None,
    }
  }
}

impl Deref for Diagnostic {
  type Target = miette::Error;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}

impl Diagnostic {
  pub fn warn(title: String, message: String) -> Self {
    Self {
      inner: Error::from(
        MietteDiagnostic::new(message)
          .with_code(title)
          .with_severity(miette::Severity::Warning),
      )
      .into(),
      details: None,
      module_identifier: None,
      loc: None,
      file: None,
      hide_stack: None,
      chunk: None,
      stack: None,
    }
  }

  pub fn error(title: String, message: String) -> Self {
    Self {
      inner: Error::from(
        MietteDiagnostic::new(message)
          .with_code(title)
          .with_severity(miette::Severity::Error),
      )
      .into(),
      details: None,
      module_identifier: None,
      loc: None,
      file: None,
      hide_stack: None,
      chunk: None,
      stack: None,
    }
  }
}

static COLORED_GRAPHICAL_REPORT_HANDLER: LazyLock<GraphicalReportHandler> = LazyLock::new(|| {
  GraphicalReportHandler::new()
    .with_theme(GraphicalTheme::unicode())
    .with_context_lines(2)
    .with_width(usize::MAX)
});

static NO_COLOR_GRAPHICAL_REPORT_HANDLER: LazyLock<GraphicalReportHandler> = LazyLock::new(|| {
  GraphicalReportHandler::new()
    .with_theme(GraphicalTheme::unicode_nocolor())
    .with_context_lines(2)
    .with_width(usize::MAX)
    .without_syntax_highlighting()
});

impl Diagnostic {
  pub fn render_report(&self, colored: bool) -> crate::Result<String> {
    let mut buf = String::new();

    let h = if colored {
      &COLORED_GRAPHICAL_REPORT_HANDLER
    } else {
      &NO_COLOR_GRAPHICAL_REPORT_HANDLER
    };

    h.render_report(&mut buf, self.as_ref()).into_diagnostic()?;
    Ok(buf)
  }

  pub fn as_miette_error(&self) -> &Arc<miette::Error> {
    &self.inner
  }

  pub fn message(&self) -> String {
    self.inner.to_string()
  }

  pub fn severity(&self) -> Severity {
    self.inner.severity().unwrap_or_default().into()
  }

  pub fn module_identifier(&self) -> Option<Identifier> {
    self.module_identifier
  }

  pub fn with_module_identifier(mut self, module_identifier: Option<Identifier>) -> Self {
    self.module_identifier = module_identifier;
    self
  }

  pub fn loc(&self) -> Option<&DependencyLocation> {
    self.loc.as_ref()
  }

  pub fn with_loc(mut self, loc: Option<DependencyLocation>) -> Self {
    self.loc = loc;
    self
  }

  pub fn file(&self) -> Option<&Utf8Path> {
    self.file.as_deref()
  }

  pub fn with_file(mut self, file: Option<Utf8PathBuf>) -> Self {
    self.file = file;
    self
  }

  pub fn hide_stack(&self) -> Option<bool> {
    self.hide_stack
  }

  pub fn with_hide_stack(mut self, hide_stack: Option<bool>) -> Self {
    self.hide_stack = hide_stack;
    self
  }

  pub fn chunk(&self) -> Option<u32> {
    self.chunk
  }

  pub fn with_chunk(mut self, chunk: Option<u32>) -> Self {
    self.chunk = chunk;
    self
  }

  pub fn stack(&self) -> Option<String> {
    self.stack.clone()
  }

  pub fn with_stack(mut self, stack: Option<String>) -> Self {
    self.stack = stack;
    self
  }

  pub fn details(&self) -> Option<String> {
    self.details.clone()
  }

  pub fn with_details(mut self, details: Option<String>) -> Self {
    self.details = details;
    self
  }
}

pub trait Diagnosable {
  fn add_diagnostic(&mut self, _diagnostic: Diagnostic);

  fn add_diagnostics(&mut self, _diagnostics: Vec<Diagnostic>);

  fn diagnostics(&self) -> Cow<'_, [Diagnostic]>;

  fn first_error(&self) -> Option<Cow<'_, Diagnostic>> {
    match self.diagnostics() {
      Cow::Borrowed(diagnostics) => diagnostics
        .iter()
        .find(|d| d.severity() == Severity::Error)
        .map(Cow::Borrowed),
      Cow::Owned(diagnostics) => diagnostics
        .into_iter()
        .find(|d| d.severity() == Severity::Error)
        .map(Cow::Owned),
    }
  }
}

#[macro_export]
macro_rules! impl_empty_diagnosable_trait {
  ($ty:ty) => {
    impl $crate::Diagnosable for $ty {
      fn add_diagnostic(&mut self, _diagnostic: $crate::Diagnostic) {
        unimplemented!(
          "`<{ty} as Diagnosable>::add_diagnostic` is not implemented",
          ty = stringify!($ty)
        )
      }
      fn add_diagnostics(&mut self, _diagnostics: Vec<$crate::Diagnostic>) {
        unimplemented!(
          "`<{ty} as Diagnosable>::add_diagnostics` is not implemented",
          ty = stringify!($ty)
        )
      }
      fn diagnostics(&self) -> std::borrow::Cow<'_, [$crate::Diagnostic]> {
        std::borrow::Cow::Owned(vec![])
      }
    }
  };
}

pub fn errors_to_diagnostics(errs: Vec<Error>) -> Vec<Diagnostic> {
  errs.into_iter().map(Diagnostic::from).collect()
}
