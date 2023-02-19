use std::sync::atomic::AtomicBool;

use tracing::Level;
use tracing_chrome::FlushGuard;
use tracing_subscriber::{fmt::format::FmtSpan, layer::Filter};
// use tracing_chrome::FlushGuard;

static IS_TRACING_ENABLED: AtomicBool = AtomicBool::new(false);
// skip event because it's not useful for performance analysis
struct FilterEvent {}
impl<S> Filter<S> for FilterEvent {
  fn enabled(
    &self,
    meta: &tracing::Metadata<'_>,
    _cx: &tracing_subscriber::layer::Context<'_, S>,
  ) -> bool {
    !meta.is_event()
  }
}
pub fn enable_tracing_by_env() {
  let is_enable_tracing = std::env::var("TRACE").map_or(false, |x| {
    matches!(x.as_str(), "TRACE" | "DEBUG" | "INFO" | "WARN" | "ERROR")
  });
  if is_enable_tracing && !IS_TRACING_ENABLED.swap(true, std::sync::atomic::Ordering::SeqCst) {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};
    tracing_subscriber::registry()
      .with(
        fmt::layer()
          .pretty()
          .with_file(true)
          // To keep track of the closing point of spans
          .with_span_events(FmtSpan::CLOSE),
      )
      .with(
        tracing_subscriber::filter::Targets::new().with_targets(vec![
          ("rspack_core", Level::TRACE),
          ("rspack", Level::TRACE),
          ("rspack_node", Level::TRACE),
          ("rspack_plugin_javascript", Level::TRACE),
          ("rspack_plugin_split_chunks", Level::TRACE),
          ("rspack_binding_options", Level::TRACE),
        ]),
      )
      // Using TRACE=[TRACE|DEBUG|INFO|WARN|ERROR] to set max trace level.
      .with(EnvFilter::from_env("TRACE"))
      .init();
    tracing::trace!("enable_tracing_by_env");
  }
}

pub fn enable_tracing_by_env_with_chrome_layer() -> Option<FlushGuard> {
  let is_enable_tracing = std::env::var("TRACE").map_or(false, |x| {
    matches!(x.as_str(), "TRACE" | "DEBUG" | "INFO" | "WARN" | "ERROR")
  });
  if is_enable_tracing && !IS_TRACING_ENABLED.swap(true, std::sync::atomic::Ordering::SeqCst) {
    use tracing_chrome::ChromeLayerBuilder;
    use tracing_subscriber::{prelude::*, EnvFilter};

    let (chrome_layer, guard) = ChromeLayerBuilder::new().include_args(true).build();
    // If we don't do this. chrome_layer will collect nothing.
    // std::mem::forget(guard);
    tracing_subscriber::registry()
      .with(chrome_layer.with_filter(FilterEvent {}))
      .with(
        tracing_subscriber::filter::Targets::new().with_targets(vec![
          ("rspack_core", Level::TRACE),
          ("rspack", Level::TRACE),
          ("rspack_node", Level::TRACE),
          ("rspack_plugin_javascript", Level::TRACE),
          ("rspack_plugin_split_chunks", Level::TRACE),
          ("rspack_binding_options", Level::TRACE),
        ]),
      )
      // Using TRACE=[TRACE|DEBUG|INFO|WARN|ERROR] to set max trace level.
      .with(EnvFilter::from_env("TRACE"))
      .init();
    Some(guard)
  } else {
    None
  }
}
