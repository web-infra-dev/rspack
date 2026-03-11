pub fn install_panic_handler() {
  static INSTALL_PANIC_HANDLER: std::sync::Once = std::sync::Once::new();

  INSTALL_PANIC_HANDLER.call_once(|| {
    let previous_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic| {
      #[cfg(debug_assertions)]
      {
        use rspack_core::debug_info::DEBUG_INFO;
        if let Ok(info) = DEBUG_INFO.lock() {
          eprintln!("{info}");
        }
      }

      eprintln!(
        "Panic occurred at runtime. Please file an issue on GitHub with the panic info and backtrace below: https://github.com/web-infra-dev/rspack/issues"
      );

      // Keep the original hook in the chain so the backtrace still points to the
      // panic site instead of this custom hook frame.
      previous_hook(panic);
    }));
  });
}
