use std::collections::HashMap;

use tracing_subscriber::{Layer, Registry};
#[derive(Debug)]
pub struct TraceEvent {
  // event name
  pub name: String,
  // separate track name
  pub track_name: Option<String>,
  // separate process name
  pub process_name: Option<String>,
  // extra debug arguments
  pub args: Option<HashMap<String, String>>,
  // track_uuid
  pub uuid: u32,
  // timestamp in microseconds
  pub ts: u64,
  // ph
  pub ph: String,
  // category
  pub categories: Option<Vec<String>>,
}

pub type Layered = Box<dyn Layer<Registry> + Send + Sync>;

pub trait Tracer {
  fn setup(&mut self, output: &str) -> Option<Layered>;
  fn sync_trace(&mut self, _events: Vec<TraceEvent>) {
    // noop
  }
  fn teardown(&mut self);
}
