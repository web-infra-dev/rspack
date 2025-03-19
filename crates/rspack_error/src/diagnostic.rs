use std::{borrow::Cow, fmt, ops::Deref, sync::Arc};

use cow_utils::CowUtils;
use miette::{GraphicalTheme, IntoDiagnostic, MietteDiagnostic};
use rspack_cacheable::{cacheable, with::Unsupported};
use rspack_collections::Identifier;
use rspack_paths::{Utf8Path, Utf8PathBuf};
use swc_core::common::{SourceMap, Span};

use crate::{graphical::GraphicalReportHandler, Error};

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

#[cacheable]
#[derive(Debug, Clone, Copy)]
pub struct ErrorLocation {
  pub start: SourcePosition,
  pub end: SourcePosition,
}

impl ErrorLocation {
  pub fn new(span: Span, source_map: &SourceMap) -> Self {
    let lo = source_map.lookup_char_pos(span.lo());
    let hi = source_map.lookup_char_pos(span.hi());

    ErrorLocation {
      start: SourcePosition {
        line: lo.line,
        column: lo.col_display,
      },
      end: SourcePosition {
        line: hi.line,
        column: hi.col_display,
      },
    }
  }
}

#[cacheable(with=Unsupported)]
#[derive(Debug, Clone)]
pub struct Diagnostic {
  inner: Arc<miette::Error>,
  module_identifier: Option<Identifier>,
  loc: Option<String>,
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

impl From<miette::Error> for Diagnostic {
  fn from(value: miette::Error) -> Self {
    Self {
      inner: Arc::new(value),
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
      module_identifier: None,
      loc: None,
      file: None,
      hide_stack: None,
      chunk: None,
      stack: None,
    }
  }
}

impl Diagnostic {
  pub fn render_report(&self, colored: bool) -> crate::Result<String> {
    let mut buf = String::new();
    let h = GraphicalReportHandler::new()
      .with_theme(if colored {
        GraphicalTheme::unicode()
      } else {
        GraphicalTheme::unicode_nocolor()
      })
      .with_context_lines(2)
      .with_width(usize::MAX);
    h.render_report(&mut buf, self.as_ref()).into_diagnostic()?;
    Ok(buf)
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

  pub fn loc(&self) -> Option<String> {
    self.loc.clone()
  }

  pub fn with_loc(mut self, loc: Option<String>) -> Self {
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
    let hide_stack = self.hide_stack.unwrap_or_default();
    if hide_stack {
      // TODO: generate detail content for typed error
      self.stack()
    } else {
      None
    }
  }
}

pub trait Diagnosable {
  fn add_diagnostic(&mut self, _diagnostic: Diagnostic);

  fn add_diagnostics(&mut self, _diagnostics: Vec<Diagnostic>);

  fn diagnostics(&self) -> Cow<[Diagnostic]>;

  fn first_error(&self) -> Option<Cow<Diagnostic>> {
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
      fn diagnostics(&self) -> std::borrow::Cow<[$crate::Diagnostic]> {
        std::borrow::Cow::Owned(vec![])
      }
    }
  };
}

pub fn errors_to_diagnostics(errs: Vec<Error>) -> Vec<Diagnostic> {
  errs.into_iter().map(Diagnostic::from).collect()
}
