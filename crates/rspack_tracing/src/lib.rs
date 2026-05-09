#[cfg(feature = "perfetto")]
mod perfetto;
mod stdout;
mod tracer;

#[cfg(feature = "perfetto")]
pub use perfetto::PerfettoTracer;
pub use stdout::StdoutTracer;
pub use tracer::{TraceEvent, Tracer};
