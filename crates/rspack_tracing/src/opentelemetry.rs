use opentelemetry::{global, trace::TracerProvider as _, KeyValue};
use opentelemetry_sdk::{propagation::TraceContextPropagator, runtime, Resource};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::Layer;

use super::tracer::Tracer;
use crate::tracer::Layered;

pub struct OtelTracer {
  provider: opentelemetry_sdk::trace::TracerProvider,
}

impl Default for OtelTracer {
  fn default() -> Self {
    Self::new()
  }
}

impl OtelTracer {
  fn new() -> Self {
    let provider =
      opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .with_trace_config(opentelemetry_sdk::trace::Config::default().with_resource(
          Resource::new(vec![KeyValue::new("service.name", "rspack-app")]),
        ))
        .install_batch(runtime::Tokio)
        .expect("Should be able to initialize open telemetry");

    Self { provider }
  }
}

impl Tracer for OtelTracer {
  fn setup(&mut self, _output: &str) -> Option<Layered> {
    global::set_text_map_propagator(TraceContextPropagator::new());
    global::set_tracer_provider(self.provider.clone());
    let trace = self.provider.tracer("rspack-app");
    Some(OpenTelemetryLayer::new(trace).boxed())
  }

  fn teardown(&mut self) {
    let _ = self.provider.shutdown();
    opentelemetry::global::shutdown_tracer_provider();
  }
}

pub mod otel {
  pub use opentelemetry;
  pub use opentelemetry_sdk as sdk;
  pub use tracing_opentelemetry as tracing;
}
