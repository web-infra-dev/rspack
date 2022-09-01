use crate::{Error, TraceableError};

#[derive(Debug, Clone, Default, Copy)]
pub enum Severity {
  #[default]
  Error,
  Warn,
}
#[derive(Debug, Clone)]
pub struct DiagnosticSourceInfo {
  pub(crate) path: String,
  pub(crate) source: String,
}
#[derive(Debug, Clone, Default)]
pub struct Diagnostic {
  pub severity: Severity,
  pub message: String,
  pub title: String,
  /// Source code and path of current Diagnostic
  pub source_info: Option<DiagnosticSourceInfo>,
  pub start: usize,
  pub end: usize,
}

impl Diagnostic {
  pub fn warn(title: String, message: String, start: usize, end: usize) -> Self {
    Self {
      severity: Severity::Warn,
      title,
      message,
      source_info: None,
      start,
      end,
    }
  }

  pub fn error(title: String, message: String, start: usize, end: usize) -> Self {
    Self {
      severity: Severity::Error,
      message,
      source_info: None,
      start,
      end,
      title,
    }
  }

  pub fn with_source_info(mut self, source: DiagnosticSourceInfo) -> Self {
    self.source_info = Some(source);
    self
  }
}

impl From<Error> for Vec<Diagnostic> {
  fn from(err: Error) -> Self {
    let diagnostic = match err {
      Error::InternalError(message) => Diagnostic {
        message,
        source_info: None,
        start: 0,
        end: 0,
        ..Default::default()
      },
      Error::TraceableError(TraceableError {
        path,
        start,
        end,
        error_message,
        source,
        title,
      }) => {
        let source = if let Some(source) = source {
          source
        } else {
          std::fs::read_to_string(&path).unwrap()
        };
        Diagnostic {
          message: error_message,
          source_info: Some(DiagnosticSourceInfo { source, path }),
          start,
          end,
          title,
          ..Default::default()
        }
      }
      Error::Io { source } => Diagnostic {
        message: source.to_string(),
        ..Default::default()
      },
      Error::Anyhow { source } => Diagnostic {
        message: format!("{}\nbacktrace: {}", source, source.backtrace()),
        ..Default::default()
      },
      Error::Json { source } => Diagnostic {
        message: source.to_string(),
        ..Default::default()
      },
      Error::BatchErrors(diagnostics) => {
        return diagnostics
          .into_iter()
          .flat_map(Vec::<Diagnostic>::from)
          .collect::<Vec<_>>()
      }
    };
    vec![diagnostic]
  }
}
