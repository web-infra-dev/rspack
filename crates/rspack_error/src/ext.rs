use std::{borrow::Cow, error::Error};

use miette::Diagnostic;

use crate::miette_helpers::{WithHelp, WithLabel};

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

pub trait MietteExt {
  fn with_help(self, message: impl Into<Cow<'static, str>>) -> Box<dyn Diagnostic + Send + Sync>;
  fn with_labels(
    self,
    labels: impl Iterator<Item = miette::LabeledSpan>,
  ) -> Box<dyn Diagnostic + Send + Sync>;
}

impl MietteExt for Box<dyn Diagnostic + Send + Sync> {
  fn with_help(self, message: impl Into<Cow<'static, str>>) -> Box<dyn Diagnostic + Send + Sync> {
    let h = WithHelp::from(self).with_help(message);
    <WithHelp as DiagnosticExt>::boxed(h)
  }
  fn with_labels(
    self,
    labels: impl Iterator<Item = miette::LabeledSpan>,
  ) -> Box<dyn Diagnostic + Send + Sync> {
    let l = WithLabel::from(self).with_label(labels);
    <WithLabel as DiagnosticExt>::boxed(l)
  }
}

impl MietteExt for miette::Error {
  fn with_help(self, message: impl Into<Cow<'static, str>>) -> Box<dyn Diagnostic + Send + Sync> {
    let h = WithHelp::from(self).with_help(message);
    <WithHelp as DiagnosticExt>::boxed(h)
  }
  fn with_labels(
    self,
    labels: impl Iterator<Item = miette::LabeledSpan>,
  ) -> Box<dyn Diagnostic + Send + Sync> {
    let l = WithLabel::from(self).with_label(labels);
    <WithLabel as DiagnosticExt>::boxed(l)
  }
}
