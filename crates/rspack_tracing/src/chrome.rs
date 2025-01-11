use tracing_chrome::{ChromeLayerBuilder, FlushGuard};
use tracing_subscriber::layer::{Filter, Layer};

use crate::{
  tracer::{Layered, Tracer},
  TraceWriter,
};

#[derive(Default)]
pub struct ChromeTracer {
  guard: Option<FlushGuard>,
}

impl Tracer for ChromeTracer {
  fn setup(&mut self, output: &str) -> Option<Layered> {
    let trace_writer = TraceWriter::from(output);
    let (chrome_layer, guard) = ChromeLayerBuilder::new()
      .include_args(true)
      .writer(trace_writer.writer())
      .build();
    self.guard = Some(guard);

    Some(vec![chrome_layer.with_filter(FilterEvent {}).boxed()].boxed())
  }

  fn teardown(&mut self) {
    if let Some(guard) = self.guard.take() {
      guard.flush();
    }
  }
}

// skip event because it's not useful for performance analysis
struct FilterEvent;

impl<S> Filter<S> for FilterEvent {
  fn enabled(
    &self,
    meta: &tracing::Metadata<'_>,
    _cx: &tracing_subscriber::layer::Context<'_, S>,
  ) -> bool {
    !meta.is_event()
  }
}
