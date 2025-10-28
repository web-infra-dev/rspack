use std::{fmt::Display, sync::Mutex};

#[cfg(allocative)]
use rspack_util::allocative;

/// Debug info used when programs panics
/// Only works with #[cfg(debug_assertions)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub struct DebugInfo {
  /// The base directory. See [options.context](https://webpack.js.org/configuration/entry-context/#context)
  pub(crate) context: Option<String>,
}

impl DebugInfo {
  const fn new() -> Self {
    Self { context: None }
  }

  pub(crate) fn with_context(&mut self, context: String) -> &mut Self {
    self.context = Some(context);
    self
  }
}

macro_rules! write_debug_info {
  ($f:ident,$tt:tt,$expr:expr) => {
    let tt = $tt.to_string();
    if let Some(e) = &$expr {
      let e = format!("\u{001b}[1m\u{001b}[33m{tt}\u{001b}[0m: {e}");
      writeln!($f, "{}", e)?;
    } else {
      let e = format!("\u{001b}[1m\u{001b}[33m{tt}\u{001b}[0m: <empty>");
      writeln!($f, "{}", e)?;
    }
  };
}

impl Display for DebugInfo {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    writeln!(f, "DebugInfo:\n")?;
    write_debug_info!(f, "context", &self.context);

    Ok(())
  }
}

#[cfg_attr(allocative, allocative::root)]
pub static DEBUG_INFO: Mutex<DebugInfo> = Mutex::new(DebugInfo::new());
