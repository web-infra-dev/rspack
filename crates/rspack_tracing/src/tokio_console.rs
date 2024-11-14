use crate::{tracer::Layered, Tracer};

pub struct TokioConsoleTracer;

impl Tracer for TokioConsoleTracer {
  fn setup(&mut self, _output: &str) -> Option<Layered> {
    console_subscriber::init();
    None
  }

  fn teardown(&mut self) {}
}
