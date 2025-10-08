use itertools::Itertools;
use rspack_error::{Diagnostic, Error, Label};

use crate::{BoxLoader, DependencyRange};

///////////////////// Module Factory /////////////////////

#[derive(Debug)]
pub struct EmptyDependency(Error);

impl EmptyDependency {
  pub fn new(span: Option<DependencyRange>) -> Self {
    let mut err = Error::error("Empty dependency: Expected a non-empty request".to_string());
    err.code = Some("Empty dependency".to_string());
    if let Some(span) = span {
      err.labels = Some(vec![Label {
        name: None,
        offset: span.start as usize,
        len: span.end.saturating_sub(span.start) as usize,
      }])
    }
    Self(err)
  }
}

impl From<EmptyDependency> for Error {
  fn from(value: EmptyDependency) -> Error {
    value.0
  }
}

///////////////////// Module /////////////////////

#[derive(Debug)]
pub struct ModuleBuildError(pub Error);

impl ModuleBuildError {
  pub fn new(error: Error) -> Self {
    Self(error)
  }
}

impl From<ModuleBuildError> for Error {
  fn from(value: ModuleBuildError) -> Error {
    let source = value.0;

    let mut err = Error::error("Module build failed:".into());
    let details = source
      .hide_stack
      .unwrap_or(false)
      .then_some(source.stack.as_ref().map(|stack| stack.to_string()))
      .flatten();
    err.details = details;
    err.severity = source.severity;
    err.source_error = Some(Box::new(source));
    err.code = Some("ModuleBuildError".into());
    err
  }
}

/// Represent any errors or warnings during module parse
///
/// This does NOT aligned with webpack as webpack does not have parse warning.
/// However, rspack may create warning during parsing stage, taking CSS as an example.
#[derive(Debug)]
pub struct ModuleParseError {
  title: &'static str,
  help: String,
  source: Error,
}

impl From<ModuleParseError> for Error {
  fn from(value: ModuleParseError) -> Error {
    let mut error = rspack_error::Error::default();
    error.message = value.title.to_string();
    error.severity = value.source.severity;
    error.code = if value.source.is_warn() {
      Some("ModuleParseWarning".into())
    } else {
      Some("ModuleParseError".into())
    };
    error.source_error = Some(Box::new(value.source));
    if !value.help.is_empty() {
      error.help = Some(value.help);
    }
    error
  }
}

impl ModuleParseError {
  pub fn new(source: Error, loaders: &[BoxLoader]) -> Self {
    let mut help = String::new();
    let mut title = "Module parse failed:";
    if source.is_error() {
      if loaders.is_empty() {
        help = format!("{help}\nYou may need an appropriate loader to handle this file type.");
      } else {
        let s = loaders
          .iter()
          .map(|l| {
            let l = l.identifier().to_string();
            format!("\n * {l}")
          })
          .join("");
        help = format!(
          "{help}\nFile was processed with these loaders:{s}\nYou may need an additional loader to handle the result of these loaders."
        );
      }
    } else {
      title = "Module parse warning:"
    }
    Self {
      title,
      help,
      source,
    }
  }
}

/// Mark boxed errors as [crate::diagnostics::ModuleParseError],
/// then, map it to diagnostics
pub fn map_box_diagnostics_to_module_parse_diagnostics(
  diagnostic: Vec<rspack_error::Diagnostic>,
  loaders: &[BoxLoader],
) -> Vec<rspack_error::Diagnostic> {
  diagnostic
    .into_iter()
    .map(|d| Error::from(ModuleParseError::new(d.error, loaders)).into())
    .collect()
}

/////////////// Minify error

#[derive(Debug)]
pub struct MinifyError(pub Error);

impl From<MinifyError> for Error {
  fn from(value: MinifyError) -> Error {
    let mut error = rspack_error::error!("Chunk minification failed:");
    error.code = if value.0.is_warn() {
      Some("ChunkMinificationWarning".into())
    } else {
      Some("ChunkMinificationError".into())
    };
    error.source_error = Some(Box::new(value.0));
    error
  }
}

impl From<MinifyError> for Diagnostic {
  fn from(value: MinifyError) -> Diagnostic {
    let error: Error = value.into();
    error.into()
  }
}
