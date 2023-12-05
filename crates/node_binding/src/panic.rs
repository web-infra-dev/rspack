use color_backtrace::{default_output_stream, BacktracePrinter};

pub fn install_panic_handler() {
  BacktracePrinter::default()
    .message("Panic occurred at runtime. Please file an issue on GitHub with the backtrace below: https://github.com/web-infra-dev/rspack/issues")
    .add_frame_filter(Box::new(|frames| {
      static NAME_PREFIXES: &[&str] = &[
        "rust_panic",
        "rayon",
        "rust_begin_unwind",
        "start_thread",
        "__clone",
        "call_once",
        "catch_unwind",
        "tokio",
        "<tokio::runtime",
        "future",
        "std::panic",
        "<core::panic",
        "___rust_try",
      ];
      static FILE_PREFIXES: &[&str] = &[
        "/rustc/",
        "src/libstd/",
        "src/libpanic_unwind/",
        "src/libtest/",
      ];
      frames.retain(|x| {
        if x.is_post_panic_code() || x.is_runtime_init_code() {
          return false;
        }

        if matches!(&x.filename, Some(f) if FILE_PREFIXES.iter().any(|l| {
          f.starts_with(l)
        })) {
          return false;
        }

        if matches!(&x.name, Some(n) if NAME_PREFIXES.iter().any(|l| {
          n.starts_with(l)
        })) {
          return false;
        }

        true
      });
    }))
    .verbosity(color_backtrace::Verbosity::Medium)
    .lib_verbosity(color_backtrace::Verbosity::Medium)
    .print_addresses(false)
    .install(default_output_stream());
}
