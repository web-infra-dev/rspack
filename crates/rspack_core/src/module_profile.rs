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
