use std::error::Error;

/// Useful to convert [std::error::Error] to [crate::DiagnosticError]
pub trait ErrorExt {
  fn boxed(self) -> Box<dyn Error + Send + Sync>;
}

impl<T: Error + Send + Sync + 'static> ErrorExt for T {
  fn boxed(self) -> Box<dyn Error + Send + Sync> {
    Box::new(self)
  }
}
