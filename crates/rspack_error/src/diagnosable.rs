use std::borrow::Cow;

use crate::diagnostic::Diagnostic;

pub trait Diagnosable {
  fn add_diagnostic(&mut self, _diagnostic: Diagnostic);

  fn add_diagnostics(&mut self, _diagnostics: Vec<Diagnostic>);

  fn diagnostics(&self) -> Cow<'_, [Diagnostic]>;

  fn first_error(&self) -> Option<Cow<'_, Diagnostic>> {
    match self.diagnostics() {
      Cow::Borrowed(diagnostics) => diagnostics.iter().find(|d| d.is_error()).map(Cow::Borrowed),
      Cow::Owned(diagnostics) => diagnostics
        .into_iter()
        .find(|d| d.is_error())
        .map(Cow::Owned),
    }
  }
}

#[macro_export]
macro_rules! impl_empty_diagnosable_trait {
  ($ty:ty) => {
    impl $crate::Diagnosable for $ty {
      fn add_diagnostic(&mut self, _diagnostic: $crate::Diagnostic) {
        unimplemented!(
          "`<{ty} as Diagnosable>::add_diagnostic` is not implemented",
          ty = stringify!($ty)
        )
      }
      fn add_diagnostics(&mut self, _diagnostics: Vec<$crate::Diagnostic>) {
        unimplemented!(
          "`<{ty} as Diagnosable>::add_diagnostics` is not implemented",
          ty = stringify!($ty)
        )
      }
      fn diagnostics(&self) -> std::borrow::Cow<'_, [$crate::Diagnostic]> {
        std::borrow::Cow::Owned(vec![])
      }
    }
  };
}
