use std::{fmt::Display, path::PathBuf};

use itertools::Itertools;
use rspack_error::{
  error, impl_diagnostic_transparent,
  miette::{self, Diagnostic},
  thiserror::{self, Error},
  DiagnosticExt, Error, TraceableError,
};
use rspack_util::ext::AsAny;
use rustc_hash::FxHashSet;

use crate::{BoxLoader, DependencyRange};

///////////////////// Module Factory /////////////////////

#[derive(Debug, Error)]
#[error(transparent)]
pub struct EmptyDependency(Box<dyn Diagnostic + Send + Sync>);

impl EmptyDependency {
  pub fn new(span: DependencyRange) -> Self {
    Self(
      TraceableError::from_lazy_file(
        span.start as usize,
        span.end as usize,
        "Empty dependency".to_string(),
        "Expected a non-empty request".to_string(),
      )
      .boxed(),
    )
  }
}

impl_diagnostic_transparent!(EmptyDependency);

///////////////////// Module /////////////////////

#[derive(Debug)]
pub struct ModuleBuildError(pub Error);

impl std::error::Error for ModuleBuildError {
  fn source(&self) -> ::core::option::Option<&(dyn std::error::Error + 'static)> {
    Some(<Error as AsRef<dyn std::error::Error>>::as_ref(&self.0))
  }
}

impl std::fmt::Display for ModuleBuildError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Module build failed:")
  }
}

impl miette::Diagnostic for ModuleBuildError {
  fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
    Some(Box::new("ModuleBuildError"))
  }
  fn severity(&self) -> Option<miette::Severity> {
    self.0.severity()
  }
  fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
    self.0.help()
  }
  fn url<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
    self.0.url()
  }
  fn source_code(&self) -> Option<&dyn miette::SourceCode> {
    self.0.source_code()
  }
  fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
    self.0.labels()
  }
  fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn miette::Diagnostic> + 'a>> {
    self.0.related()
  }
  fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
    Some(self.0.as_ref())
  }
}

/// Represent any errors or warnings during module parse
///
/// This does NOT aligned with webpack as webpack does not have parse warning.
/// However, rspack may create warning during parsing stage, taking CSS as an example.
#[derive(Debug, Error)]
#[error("{title}")]
pub struct ModuleParseError {
  message: String,
  title: &'static str,
  help: String,
  #[source]
  source: Box<dyn Diagnostic + Send + Sync>,
}

impl miette::Diagnostic for ModuleParseError {
  // Passthrough the severity
  fn severity(&self) -> Option<miette::Severity> {
    self.source.severity()
  }

  fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
    match self.severity().unwrap_or(miette::Severity::Error) {
      miette::Severity::Advice => unreachable!("miette::Severity::Advice should not be used"),
      miette::Severity::Warning => Some(Box::new("ModuleParseWarning")),
      miette::Severity::Error => Some(Box::new("ModuleParseError")),
    }
  }

  fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
    if self.help.is_empty() {
      return None;
    }
    Some(Box::new(&self.help))
  }

  fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
    Some(&*self.source)
  }
}

impl ModuleParseError {
  pub fn new(source: Box<dyn Diagnostic + Send + Sync>, loaders: &[BoxLoader]) -> Self {
    let message = source.to_string();
    let mut help = String::new();
    let mut title = "Module parse failed:";
    if source.severity().unwrap_or(miette::Severity::Error) >= miette::Severity::Error {
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
        help = format!("{help}\nFile was processed with these loaders:{s}\nYou may need an additional loader to handle the result of these loaders.");
      }
    } else {
      title = "Module parse warning:"
    }
    Self {
      message,
      title,
      help,
      source,
    }
  }
}

#[derive(Debug)]
pub struct CapturedLoaderError {
  pub message: String,
  pub stack: Option<String>,
  pub hide_stack: Option<bool>,
  pub file_dependencies: Vec<String>,
  pub context_dependencies: Vec<String>,
  pub missing_dependencies: Vec<String>,
  pub build_dependencies: Vec<String>,
}

impl CapturedLoaderError {
  pub fn new(
    message: String,
    stack: Option<String>,
    hide_stack: Option<bool>,
    file_dependencies: Vec<String>,
    context_dependencies: Vec<String>,
    missing_dependencies: Vec<String>,
    build_dependencies: Vec<String>,
  ) -> Self {
    Self {
      message,
      stack,
      hide_stack,
      file_dependencies,
      context_dependencies,
      missing_dependencies,
      build_dependencies,
    }
  }

  pub fn take_message(&mut self) -> String {
    std::mem::take(&mut self.message)
  }

  pub fn take_stack(&mut self) -> Option<String> {
    std::mem::take(&mut self.stack)
  }

  pub fn take_file_dependencies(&mut self) -> FxHashSet<PathBuf> {
    std::mem::take(&mut self.file_dependencies)
      .into_iter()
      .map(Into::into)
      .collect()
  }

  pub fn take_context_dependencies(&mut self) -> FxHashSet<PathBuf> {
    std::mem::take(&mut self.context_dependencies)
      .into_iter()
      .map(Into::into)
      .collect()
  }

  pub fn take_missing_dependencies(&mut self) -> FxHashSet<PathBuf> {
    std::mem::take(&mut self.missing_dependencies)
      .into_iter()
      .map(Into::into)
      .collect()
  }

  pub fn take_build_dependencies(&mut self) -> FxHashSet<PathBuf> {
    std::mem::take(&mut self.build_dependencies)
      .into_iter()
      .map(Into::into)
      .collect()
  }
}

impl std::error::Error for CapturedLoaderError {
  fn source(&self) -> ::core::option::Option<&(dyn std::error::Error + 'static)> {
    None
  }
}

impl std::fmt::Display for CapturedLoaderError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "CapturedLoaderError: {}", self.message)
  }
}

impl rspack_error::miette::Diagnostic for CapturedLoaderError {
  fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
    Some(Box::new("CapturedLoaderError"))
  }
}

/// Mark boxed errors as [crate::diagnostics::ModuleParseError],
/// then, map it to diagnostics
pub fn map_box_diagnostics_to_module_parse_diagnostics(
  errors: Vec<Box<dyn Diagnostic + Send + Sync + 'static>>,
  loaders: &[BoxLoader],
) -> Vec<rspack_error::Diagnostic> {
  errors
    .into_iter()
    .map(|e| {
      let hide_stack = e
        .as_any()
        .downcast_ref::<TraceableError>()
        .and_then(|e| e.hide_stack());
      let diagnostic: rspack_error::Diagnostic =
        rspack_error::miette::Error::new(ModuleParseError::new(e, loaders)).into();
      diagnostic.with_hide_stack(hide_stack)
    })
    .collect()
}
