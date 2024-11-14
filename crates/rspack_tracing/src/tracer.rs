use tracing_subscriber::{Layer, Registry};

pub type Layered = Box<dyn Layer<Registry> + Send + Sync>;

pub trait Tracer {
  fn setup(&mut self, output: &str) -> Option<Layered>;
  fn teardown(&mut self);
}
