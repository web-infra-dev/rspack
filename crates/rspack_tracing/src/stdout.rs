use std::io::Write;

use tracing_subscriber::fmt::format::FmtSpan;

use crate::{
  TraceEvent, TraceWriter,
  tracer::{Layered, Tracer},
};

pub struct StdoutTracer {
  writer: Option<Box<dyn Write + Send>>,
}

impl Default for StdoutTracer {
  fn default() -> Self {
    Self { writer: None }
  }
}

impl Tracer for StdoutTracer {
  fn setup(&mut self, output: &str) -> Option<Layered> {
    use tracing_subscriber::{fmt, prelude::*};
    let trace_writer = TraceWriter::from(output.to_owned());

    // Store a clone of the writer for sync_trace
    self.writer = match output {
      "stdout" => Some(Box::new(std::io::stdout())),
      "stderr" => Some(Box::new(std::io::stderr())),
      path => {
        let file = std::fs::File::create(path)
          .unwrap_or_else(|e| panic!("Failed to create trace file: {path} due to {e}"));
        Some(Box::new(file))
      }
    };

    Some(
      fmt::layer()
        .json() // Use JSON format for structured logging for easier parsing and debugging
        .with_file(false)
        // To keep track of the closing point of spans
        .with_span_events(FmtSpan::CLOSE)
        .with_writer(trace_writer.make_writer())
        .boxed(),
    )
  }

  fn sync_trace(&mut self, events: Vec<TraceEvent>) {
    if let Some(writer) = &mut self.writer {
      for event in events {
        // Serialize TraceEvent to JSON, including track_name and process_name
        let json_value = serde_json::json!({
          "name": event.name,
          "track_name": event.track_name,
          "process_name": event.process_name,
          "ts": event.ts,
          "ph": event.ph,
          "uuid": event.uuid,
          "args": event.args,
          "categories": event.categories,
        });

        // Write as JSON line
        if let Ok(json_str) = serde_json::to_string(&json_value) {
          let _ = writeln!(writer, "{}", json_str);
        }
      }
      // Flush to ensure events are written immediately
      let _ = writer.flush();
    }
  }

  fn teardown(&mut self) {
    // Flush any remaining data
    if let Some(writer) = &mut self.writer {
      let _ = writer.flush();
    }
  }
}
