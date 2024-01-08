use std::error::Error;

use miette::Diagnostic;

/// Useful to convert [std::error::Error] to [crate::DiagnosticError]
pub trait ErrorExt {
  fn boxed(self) -> Box<dyn Error + Send + Sync>;
}

impl<T: Error + Send + Sync + 'static> ErrorExt for T {
  fn boxed(self) -> Box<dyn Error + Send + Sync> {
    Box::new(self)
  }
}

pub trait DiagnosticExt {
  fn boxed(self) -> Box<dyn Diagnostic + Send + Sync>;
}

impl<T: Diagnostic + Send + Sync + 'static> DiagnosticExt for T {
  fn boxed(self) -> Box<dyn Diagnostic + Send + Sync> {
    Box::new(self)
  }
}
