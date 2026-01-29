use std::{
  io::Write,
  sync::{Arc, Mutex},
};

use tracing_subscriber::fmt::{MakeWriter, format::FmtSpan};

use crate::{
  TraceEvent,
  tracer::{Layered, Tracer},
};

// A custom MakeWriter that wraps a shared Arc<Mutex<>> writer
#[derive(Clone)]
struct SharedWriterMaker {
  writer: Arc<Mutex<dyn Write + Send>>,
}

// Wrapper to implement Write trait for the MakeWriter
struct SharedWriter {
  writer: Arc<Mutex<dyn Write + Send>>,
}

impl Write for SharedWriter {
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    self
      .writer
      .lock()
      .expect("Failed to lock writer")
      .write(buf)
  }

  fn flush(&mut self) -> std::io::Result<()> {
    self.writer.lock().expect("Failed to lock writer").flush()
  }
}

impl<'a> MakeWriter<'a> for SharedWriterMaker {
  type Writer = SharedWriter;

  fn make_writer(&'a self) -> Self::Writer {
    SharedWriter {
      writer: self.writer.clone(),
    }
  }
}

/// Converts a microsecond timestamp to ISO 8601 format with microsecond precision
/// Example: 1704708707916028 -> "2026-01-08T11:31:47.916028Z"
fn format_timestamp_iso8601(micros: u64) -> String {
  use chrono::{DateTime, Utc};

  let secs = (micros / 1_000_000) as i64;
  let subsec_micros = (micros % 1_000_000) as u32;
  let nanos = subsec_micros * 1000;

  DateTime::<Utc>::from_timestamp(secs, nanos).map_or_else(|| "Invalid timestamp".to_string(), |dt| dt.to_rfc3339_opts(chrono::SecondsFormat::Micros, true))
}

#[derive(Default)]
pub struct StdoutTracer {
  begin_ts: u64,
  writer: Option<Arc<Mutex<dyn Write + Send>>>,
}

impl Tracer for StdoutTracer {
  fn setup(&mut self, output: &str) -> Option<Layered> {
    use tracing_subscriber::{fmt, prelude::*};

    // Record the start time in microseconds since UNIX epoch
    self.begin_ts = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .expect("System time before UNIX epoch")
      .as_micros() as u64;

    // Create the shared writer wrapped in Arc<Mutex<>>
    let writer: Arc<Mutex<Box<dyn Write + Send>>> = match output {
      "stdout" => Arc::new(Mutex::new(Box::new(std::io::stdout()))),
      "stderr" => Arc::new(Mutex::new(Box::new(std::io::stderr()))),
      path => {
        let file = std::fs::File::create(path)
          .unwrap_or_else(|e| panic!("Failed to create trace file: {path} due to {e}"));
        Arc::new(Mutex::new(Box::new(file)))
      }
    };

    // Store the shared writer for sync_trace
    self.writer = Some(writer.clone());

    // Create a custom MakeWriter that uses the same shared writer
    let make_writer = SharedWriterMaker { writer };

    Some(
      fmt::layer()
        .json() // Use JSON format for structured logging for easier parsing and debugging
        .with_file(false)
        // To keep track of the closing point of spans
        .with_span_events(FmtSpan::CLOSE)
        .with_writer(make_writer)
        .boxed(),
    )
  }

  fn sync_trace(&mut self, events: Vec<TraceEvent>) {
    if let Some(writer) = &self.writer {
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
              let span_obj = begin_event.track_name.map(|track_name| {
                serde_json::json!({
                  "name": track_name,
                })
              });

              // Convert relative microsecond timestamp to absolute ISO 8601 format
              let absolute_ts_micros = self.begin_ts + begin_event.ts;
              let timestamp_iso = format_timestamp_iso8601(absolute_ts_micros);

              // Build JSON in Rust trace format
              let json_value = serde_json::json!({
                "timestamp": timestamp_iso,
                "level": "DEBUG",
                "fields": fields,
                "target": "javascript",
                "span": span_obj,
              });

              if let Ok(json_str) = serde_json::to_string(&json_value) {
                // Lock the mutex to access the writer
                let _ = writeln!(
                  writer.lock().expect("Failed to lock writer"),
                  "{json_str}"
                );
              }
            }
          }
          _ => {}
        }
      }

      // Flush to ensure events are written immediately
      let _ = writer.lock().expect("Failed to lock writer").flush();
    }
  }

  fn teardown(&mut self) {
    // Flush any remaining data
    if let Some(writer) = &self.writer {
      let _ = writer.lock().expect("Failed to lock writer").flush();
    }
  }
}
