mod perfetto;
mod stdout;
mod tracer;

use std::{fs, io, path::PathBuf};

pub use perfetto::PerfettoTracer;
pub use stdout::StdoutTracer;
pub use tracer::{TraceEvent, Tracer};
use tracing_subscriber::fmt::writer::BoxMakeWriter;
pub(crate) enum TraceWriter {
  Stdout,
  Stderr,
  File { path: PathBuf },
}

impl From<String> for TraceWriter {
  fn from(s: String) -> Self {
    match s.as_str() {
      "stdout" => Self::Stdout,
      "stderr" => Self::Stderr,
      _ => Self::File { path: s.into() },
    }
  }
}

impl TraceWriter {
  pub fn make_writer(&self) -> BoxMakeWriter {
    match self {
      TraceWriter::Stdout => BoxMakeWriter::new(io::stdout),
      TraceWriter::Stderr => BoxMakeWriter::new(io::stderr),
      TraceWriter::File { path } => {
        BoxMakeWriter::new(fs::File::create(path).expect("Failed to create trace file"))
      }
    }
  }
}
