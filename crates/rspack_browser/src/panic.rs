//! See [install_panic_handler]

/// Install panic handler for browser environment
///
/// Rust builtin panic handler or other community implementation always lock backtrace and stderr before printing error messages.
/// However, requesting locks finally calls `atomics.wait`, and it's forbidden to do it in the main thread of browser.
/// So this function provides a panic handler which directly writes error messages to stderr without locks.
pub fn install_panic_handler() {
  use std::{fs::File, io::Write, os::fd::FromRawFd};

  std::panic::set_hook(Box::new(|info| {
    let msg = info.to_string();
    // SAFETY: only use this in browser, where fd 2 always points to `console.err`
    let mut file = unsafe { File::from_raw_fd(2) };
    file
      .write_all(msg.as_bytes())
      .expect("Failed to write error messages");
  }));
}
