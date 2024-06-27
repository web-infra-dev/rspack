use std::time::{Duration, Instant};

use once_cell::sync::OnceCell;

#[derive(Debug, Default, Clone)]
pub struct TimeRange {
  start: OnceCell<Instant>,
  end: OnceCell<Instant>,
}

impl TimeRange {
  pub fn with_value(start: Instant, end: Instant) -> Self {
    Self {
      start: OnceCell::with_value(start),
      end: OnceCell::with_value(end),
    }
  }

  pub fn duration(&self) -> Option<Duration> {
    if let Some(end) = self.end.get()
      && let Some(start) = self.start.get()
    {
      Some(end.duration_since(*start))
    } else {
      None
    }
  }
}

#[derive(Debug, Default, Clone)]
pub struct ModulePhaseProfile {
  range: TimeRange,
  parallelism_factor: OnceCell<u16>,
}

impl ModulePhaseProfile {
  pub fn duration(&self) -> Option<Duration> {
    self.range.duration()
  }

  pub fn set_parallelism_factor(&self, factor: u16) {
    self
      .parallelism_factor
      .set(factor)
      .expect("should only call once");
  }
}

// https://github.com/webpack/webpack/blob/4809421990a20dfefa06e6445191e65001e75f88/lib/ModuleProfile.js
// NOTE: Rspack has different cache design, remove cache related profiles

#[derive(Debug, Default, Clone)]
pub struct ModuleProfile {
  pub factory: ModulePhaseProfile,
  pub building: ModulePhaseProfile,
}

impl ModuleProfile {
  pub fn mark_factory_start(&self) {
    self
      .factory
      .range
      .start
      .set(Instant::now())
      .expect("should only call once");
  }

  pub fn mark_factory_end(&self) {
    self
      .factory
      .range
      .end
      .set(Instant::now())
      .expect("should only call once");
  }

  pub fn mark_building_start(&self) {
    self
      .building
      .range
      .start
      .set(Instant::now())
      .expect("should only call once");
  }

  pub fn mark_building_end(&self) {
    self
      .building
      .range
      .end
      .set(Instant::now())
      .expect("should only call once");
  }
}
