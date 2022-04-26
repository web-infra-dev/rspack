use std::sync::atomic::AtomicBool;

static IS_TRACING_ENABLED: AtomicBool = AtomicBool::new(false);

pub fn enable_tracing_by_env() {
  let is_enable_tracing = std::env::var("TRACE").map_or(false, |x| {
    matches!(x.as_str(), "TRACE" | "DEBUG" | "INFO" | "WARN" | "ERROR")
  });
  if is_enable_tracing {
    if !IS_TRACING_ENABLED.swap(true, std::sync::atomic::Ordering::SeqCst) {
      use tracing_chrome::ChromeLayerBuilder;
      use tracing_subscriber::{fmt, prelude::*, EnvFilter};

      let (chrome_layer, guard) = ChromeLayerBuilder::new().build();
      // If we don't do this. chrome_layer will collect nothing.
      std::mem::forget(guard);
      tracing_subscriber::registry()
        .with(chrome_layer)
        .with(fmt::layer().pretty().with_file(false))
        // Using TRACE=[TRACE|DEBUG|INFO|WARN|ERROR] to set max trace level.
        .with(EnvFilter::from_env("TRACE"))
        .init();
    }
  }
}
