use rspack_error::miette::{self, Error};

#[derive(Debug)]
pub struct BuiltinSwcLoaderError(pub Error);

impl std::error::Error for BuiltinSwcLoaderError {
  fn source(&self) -> ::core::option::Option<&(dyn std::error::Error + 'static)> {
    Some(<Error as AsRef<dyn std::error::Error>>::as_ref(&self.0))
  }
}

impl std::fmt::Display for BuiltinSwcLoaderError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "builtin:swc-loader Error:")
  }
}

impl miette::Diagnostic for BuiltinSwcLoaderError {
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
  fn diagnostic_source(&self) -> Option<&dyn miette::Diagnostic> {
    Some(self.0.as_ref())
  }
}
