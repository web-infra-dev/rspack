use rustc_hash::FxHashMap as HashMap;
use tracing::span::EnteredSpan;

#[napi]
pub struct Trace {
  active_spans: HashMap<String, EnteredSpan>,
}

#[napi]
impl Trace {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      active_spans: HashMap::default(),
    }
  }

  #[napi]
  pub fn time(&mut self, label: String) {
    if let Some(span) = self.active_spans.remove(&label) {
      span.exit();
    }
    let span = tracing::span!(tracing::Level::INFO, "js", label);
    let span = span.entered();
    self.active_spans.insert(label, span);
  }

  #[napi]
  pub fn time_end(&mut self, label: String) {
    if let Some(span) = self.active_spans.remove(&label) {
      span.exit();
    }
  }
}
