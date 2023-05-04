use std::str::FromStr;
use std::sync::atomic::AtomicBool;

use tracing::Level;
use tracing_chrome::FlushGuard;
use tracing_subscriber::filter;
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
pub fn enable_tracing_by_env() -> Option<FlushGuard> {
  let trace_var = std::env::var("TRACE");
  let is_enable_tracing = trace_var.is_ok();
  // Sometimes developer may want to trace the upstream lib events,
  // by default, we only trace the event emitted by rspack
  let full_tracing = std::env::var("FULL_TRACING").is_ok();
  if is_enable_tracing && !IS_TRACING_ENABLED.swap(true, std::sync::atomic::Ordering::SeqCst) {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};
    let default_layer = trace_var
      .ok()
      .as_ref()
      .and_then(|var| Level::from_str(var).ok())
      .unwrap_or(Level::TRACE);
    tracing_subscriber::registry()
      .with(
        tracing_subscriber::filter::Targets::new()
          .with_targets(vec![
            ("rspack_core", default_layer),
            ("rspack", default_layer),
            ("rspack_node", default_layer),
            ("rspack_plugin_javascript", default_layer),
            ("rspack_plugin_split_chunks", default_layer),
            ("rspack_binding_options", default_layer),
          ])
          .with_filter(filter::filter_fn(move |_| full_tracing)),
      )
      .with(EnvFilter::from_env("TRACE"))
      .with(
        fmt::layer()
          .pretty()
          .with_file(true)
          // To keep track of the closing point of spans
          .with_span_events(FmtSpan::CLOSE),
      )
      .init();
    tracing::trace!("enable_tracing_by_env");
  }
  None
}

pub fn enable_tracing_by_env_with_chrome_layer() -> Option<FlushGuard> {
  let trace_var = std::env::var("TRACE");
  let is_enable_tracing = trace_var.is_ok();
  // Sometimes developer may want to trace the upstream lib events,
  // by default, we only trace the event emitted by rspack
  let full_tracing = std::env::var("FULL_TRACING").is_ok();
  if is_enable_tracing && !IS_TRACING_ENABLED.swap(true, std::sync::atomic::Ordering::SeqCst) {
    use tracing_chrome::ChromeLayerBuilder;
    use tracing_subscriber::{prelude::*, EnvFilter};

    let (chrome_layer, guard) = ChromeLayerBuilder::new().include_args(true).build();
    // If we don't do this. chrome_layer will collect nothing.
    // std::mem::forget(guard);
    let default_layer = trace_var
      .ok()
      .as_ref()
      .and_then(|var| Level::from_str(var).ok())
      .unwrap_or(Level::TRACE);
    tracing_subscriber::registry()
      .with(
        tracing_subscriber::filter::Targets::new()
          .with_targets(vec![
            ("rspack_core", default_layer),
            ("rspack", default_layer),
            ("rspack_node", default_layer),
            ("rspack_plugin_javascript", default_layer),
            ("rspack_plugin_split_chunks", default_layer),
            ("rspack_binding_options", default_layer),
          ])
          .with_filter(filter::filter_fn(move |_| full_tracing)),
      )
      .with(EnvFilter::from_env("TRACE"))
      .with(chrome_layer.with_filter(FilterEvent {}))
      .init();
    Some(guard)
  } else {
    None
  }
}
