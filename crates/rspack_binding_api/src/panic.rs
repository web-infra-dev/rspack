pub fn install_panic_handler() {
  std::panic::set_hook(Box::new(|panic| {
    #[cfg(debug_assertions)]
    {
      use rspack_core::debug_info::DEBUG_INFO;
      if let Ok(info) = DEBUG_INFO.lock() {
        eprintln!("{info}");
      }
    }

    let backtrace = std::backtrace::Backtrace::force_capture();
    eprintln!(
      "Panic occurred at runtime. Please file an issue on GitHub with the panic info and backtrace below: https://github.com/web-infra-dev/rspack/issues"
    );
    eprintln!("panic: {panic}");
    eprintln!("backtrace:\n{backtrace}");
  }));
}
