#![feature(let_chains)]
#![feature(anonymous_lifetime_in_impl_trait)]

mod catch_unwind;
mod diagnostic;
mod error;
mod ext;
pub(crate) mod graphical;
pub(crate) mod miette_helpers;
pub use catch_unwind::*;
pub use diagnostic::*;
pub use error::*;
pub use ext::*;
pub mod emitter;

mod macros;

pub use miette;
pub use thiserror;

pub type Error = miette::Error;

pub type Result<T, E = miette::Error> = std::result::Result<T, E>;

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

#[doc(hidden)]
pub mod __private {
  pub use core::result::Result::Err;

  pub use miette::miette;
  pub use miette::Severity;

  pub use crate::diagnostic::Severity as RspackSeverity;
  pub use crate::error;
  pub use crate::error::InternalError;
}
