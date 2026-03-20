use rspack_cacheable::cacheable;

fn now_ms() -> u64 {
  std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap_or_default()
    .as_millis() as u64
}

/// Timing record for a single loader's pitch and normal phases.
/// Fields are milliseconds since UNIX epoch; 0 means the phase was not executed.
#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct LoaderTimingRecord {
  /// The loader's full request identifier (path + query + fragment).
  pub identifier: String,
  pub pitch_start_at: u64,
  pub pitch_end_at: u64,
  pub normal_start_at: u64,
  pub normal_end_at: u64,
}

impl LoaderTimingRecord {
  pub fn new(identifier: String) -> Self {
    Self {
      identifier,
      ..Default::default()
    }
  }

  pub fn record_pitch_start(&mut self) {
    self.pitch_start_at = now_ms();
  }

  pub fn record_pitch_end(&mut self) {
    self.pitch_end_at = now_ms();
  }

  pub fn record_normal_start(&mut self) {
    self.normal_start_at = now_ms();
  }

  pub fn record_normal_end(&mut self) {
    self.normal_end_at = now_ms();
  }
}
