use tracing_subscriber::fmt::format::FmtSpan;

use crate::{
  tracer::{Layered, Tracer},
  TraceWriter,
};

pub struct StdoutTracer;

impl Tracer for StdoutTracer {
  fn setup(&mut self, output: &str) -> Option<Layered> {
    use tracing_subscriber::{fmt, prelude::*};
    let trace_writer = TraceWriter::from(output.to_owned());
    Some(
      fmt::layer()
        .json() // Use JSON format for structured logging for easier parsing and debugging
        .with_file(true)
        // To keep track of the closing point of spans
        .with_span_events(FmtSpan::CLOSE)
        .with_writer(trace_writer.make_writer())
        .boxed(),
    )
  }

  fn teardown(&mut self) {
    // noop
  }
}
