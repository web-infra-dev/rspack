//! This module provides a trait for creating custom syntax highlighters that
//! highlight [`Diagnostic`](crate::Diagnostic) source code with ANSI escape
//! sequences when rendering with the [`GraphicalReportHighlighter`](crate::graphical::GraphicalReportHandler).
//!
//! It also provides built-in highlighter implementations that you can use out of the box.
//! By default, there are no syntax highlighters exported by miette
//! (except for the no-op [`BlankHighlighter`]).
//! To enable support for specific highlighters, you should enable their associated feature flag.
//!
//! Currently supported syntax highlighters and their feature flags:
//! * `syntect-highlighter` - Enables [`syntect`](https://docs.rs/syntect/latest/syntect/) syntax highlighting support via the [`SyntectHighlighter`]

/// THIS FILE IS ORIGINALLY FROM THE MIETTE PROJECT:
/// https://github.com/zkat/miette/blob/907857058dc255caeae456e87146c629ce69cf5c/src/highlighters/mod.rs
use std::{ops::Deref, sync::Arc};

#[cfg(not(target_family = "wasm"))]
use miette::highlighters::SyntectHighlighter;
use miette::highlighters::{BlankHighlighter, Highlighter};

/// Arcified trait object for Highlighter. Used internally by [`crate::graphical::GraphicalReportHandler`]
///
/// Wrapping the trait object in this way allows us to implement `Debug` and `Clone`.
#[derive(Clone)]
#[repr(transparent)]
pub(crate) struct MietteHighlighter(Arc<dyn Highlighter + Send + Sync>);

impl MietteHighlighter {
  pub(crate) fn nocolor() -> Self {
    Self::from(BlankHighlighter)
  }

  #[cfg(not(target_family = "wasm"))]
  pub(crate) fn syntect_truecolor() -> Self {
    Self::from(SyntectHighlighter::default())
  }
}

impl Default for MietteHighlighter {
  #[cfg(not(target_family = "wasm"))]
  fn default() -> Self {
    use std::io::IsTerminal;
    match std::env::var("NO_COLOR") {
      _ if !std::io::stdout().is_terminal() || !std::io::stderr().is_terminal() => {
        //TODO: should use ANSI styling instead of 24-bit truecolor here
        MietteHighlighter::syntect_truecolor()
      }
      Ok(string) if string != "0" => MietteHighlighter::nocolor(),
      _ => MietteHighlighter::syntect_truecolor(),
    }
  }

  #[cfg(target_family = "wasm")]
  fn default() -> Self {
    MietteHighlighter::nocolor()
  }
}

impl<T: Highlighter + Send + Sync + 'static> From<T> for MietteHighlighter {
  fn from(value: T) -> Self {
    Self(Arc::new(value))
  }
}

impl std::fmt::Debug for MietteHighlighter {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "MietteHighlighter(...)")
  }
}

impl Deref for MietteHighlighter {
  type Target = dyn Highlighter + Send + Sync;
  fn deref(&self) -> &Self::Target {
    &*self.0
  }
}
