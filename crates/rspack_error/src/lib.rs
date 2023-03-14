#![feature(let_chains)]
#![feature(anonymous_lifetime_in_impl_trait)]

mod catch_unwind;
mod diagnostic;
mod error;
pub use catch_unwind::*;
pub use diagnostic::*;
pub use error::*;
pub mod emitter;

mod macros;

pub type Result<T> = std::result::Result<T, Error>;

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

  pub fn split_into_parts(mut self) -> (T, Vec<Diagnostic>) {
    let diagnostic = std::mem::take(&mut self.diagnostic);
    (self.inner, diagnostic)
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

#[doc(hidden)]
pub mod __private {
  pub use core::result::Result::Err;

  pub use crate::error::{Error, InternalError};
  pub use crate::internal_error;
}
