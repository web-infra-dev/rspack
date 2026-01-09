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
      use std::collections::HashMap;

      // Track begin events by uuid to match with end events
      let mut pending_events: HashMap<u32, TraceEvent> = HashMap::new();

      for event in events {
        match event.ph.as_str() {
          "b" => {
            // Store begin event
            pending_events.insert(event.uuid, event);
          }
          "e" => {
            // Find matching begin event and calculate duration
            if let Some(begin_event) = pending_events.remove(&event.uuid) {
              let duration_ns = event.ts.saturating_sub(begin_event.ts);
              let duration_ms = duration_ns as f64 / 1_000_000.0;

              // Build fields object
              let mut fields = serde_json::Map::new();
              fields.insert("message".to_string(), serde_json::json!("close"));
              fields.insert(
                "time.busy".to_string(),
                serde_json::json!(format!("{:.2}ms", duration_ms)),
              );

              // Add any args from the event
              if let Some(args) = begin_event.args {
                for (key, value) in args {
                  fields.insert(key, serde_json::json!(value));
                }
              }

              // Build span object if we have track_name
              let span_obj = if let Some(track_name) = &begin_event.track_name {
                Some(serde_json::json!({
                  "name": track_name,
                }))
              } else {
                None
              };

              // Build JSON in Rust trace format
              // ts is in nanoseconds, convert to microseconds for chrono
              let json_value = serde_json::json!({
                "level": "DEBUG",
                "fields": fields,
                "target": begin_event.process_name.as_deref().unwrap_or("javascript"),
                "span": span_obj,
              });

              if let Ok(json_str) = serde_json::to_string(&json_value) {
                let _ = writeln!(writer, "{}", json_str);
              }
            }
          }
          _ => {}
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
