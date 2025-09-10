use crate::diagnostic::Diagnostic;

/// A helper struct for change logic from
/// return something to something with diagnostics array
#[derive(Debug)]
pub struct TWithDiagnosticArray<T: std::fmt::Debug> {
  pub inner: T,
  pub diagnostic: Vec<Diagnostic>,
}

impl<T: std::fmt::Debug> TWithDiagnosticArray<T> {
  pub fn new(inner: T, diagnostic: Vec<Diagnostic>) -> Self {
    Self { inner, diagnostic }
  }

  pub fn diagnostics(&self) -> &Vec<Diagnostic> {
    &self.diagnostic
  }

  pub fn take_inner(self) -> T {
    self.inner
  }

  pub fn split_into_parts(self) -> (T, Vec<Diagnostic>) {
    (self.inner, self.diagnostic)
  }

  pub fn get(&self) -> &T {
    &self.inner
  }
}

impl<T: Clone + std::fmt::Debug> Clone for TWithDiagnosticArray<T> {
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
      diagnostic: self.diagnostic.clone(),
    }
  }
}

// Helper trait to make `TWithDiagnosticArray` conversion more easily.
pub trait IntoTWithDiagnosticArray {
  fn with_diagnostic(self, diagnostic: Vec<Diagnostic>) -> TWithDiagnosticArray<Self>
  where
    Self: Sized + std::fmt::Debug;

  fn with_empty_diagnostic(self) -> TWithDiagnosticArray<Self>
  where
    Self: Sized + std::fmt::Debug,
  {
    TWithDiagnosticArray {
      inner: self,
      diagnostic: vec![],
    }
  }
}

impl<T: Sized + std::fmt::Debug> IntoTWithDiagnosticArray for T {
  fn with_diagnostic(self, diagnostic: Vec<Diagnostic>) -> TWithDiagnosticArray<Self>
  where
    Self: Sized + std::fmt::Debug,
  {
    TWithDiagnosticArray {
      inner: self,
      diagnostic,
    }
  }
}
