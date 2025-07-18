use rspack_error::miette::{Diagnostic, Error, Severity};

#[derive(Debug)]
pub struct BuildDependencyError(Error);

impl BuildDependencyError {
  pub fn new(err: Error) -> Self {
    Self(err)
  }
}

impl std::error::Error for BuildDependencyError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    Some(<Error as AsRef<dyn std::error::Error>>::as_ref(&self.0))
  }
}

impl std::fmt::Display for BuildDependencyError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "cache.buildDependencies error:")
  }
}

impl Diagnostic for BuildDependencyError {
  fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
    Some(Box::new("CacheBuildDependencyError"))
  }
  fn severity(&self) -> Option<Severity> {
    Some(Severity::Warning)
  }
  fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
    Some(self.0.as_ref())
  }
}
