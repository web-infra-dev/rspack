mod chrome;
mod opentelemetry;
mod stdout;
mod tracer;

use std::{fs, io, path::Path};

pub use chrome::ChromeTracer;
pub use opentelemetry::OtelTracer;
pub use stdout::StdoutTracer;
pub use tracer::Tracer;
use tracing_subscriber::fmt::writer::BoxMakeWriter;

pub mod otel {
  pub use opentelemetry;
  pub use opentelemetry_sdk as sdk;
  pub use tracing_opentelemetry as tracing;
}

pub(crate) enum TraceWriter<'a> {
  Stdout,
  Stderr,
  File { path: &'a Path },
}

impl<'a> From<&'a str> for TraceWriter<'a> {
  fn from(s: &'a str) -> Self {
    match s {
      "stdout" => Self::Stdout,
      "stderr" => Self::Stderr,
      path => Self::File {
        path: Path::new(path),
      },
    }
  }
}

impl TraceWriter<'_> {
  pub fn make_writer(&self) -> BoxMakeWriter {
    match self {
      TraceWriter::Stdout => BoxMakeWriter::new(io::stdout),
      TraceWriter::Stderr => BoxMakeWriter::new(io::stderr),
      TraceWriter::File { path } => {
        BoxMakeWriter::new(fs::File::create(path).expect("Failed to create trace file"))
      }
    }
  }

  pub fn writer(&self) -> Box<dyn io::Write + Send + 'static> {
    match self {
      TraceWriter::Stdout => Box::new(io::stdout()),
      TraceWriter::Stderr => Box::new(io::stderr()),
      TraceWriter::File { path } => {
        Box::new(fs::File::create(path).expect("Failed to create trace file"))
      }
    }
  }
}
