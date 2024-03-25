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

#[derive(Debug, Default, Clone)]
pub struct ModuleProfile {
  pub factory: ModulePhaseProfile,
  // pub restoring: ModulePhaseProfile,
  pub integration: ModulePhaseProfile,
  pub building: ModulePhaseProfile,
  // pub storing: ModulePhaseProfile,

  // pub additional_factory_times: Vec<TimeRange>,
  // pub additional_factories: Duration,
  // pub additional_factories_parallelism_factor: u16,
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

  // TODO: restore module to cache is not implemented yet
  // pub fn mark_restoring_start(&self) {
  //   self
  //     .restoring
  //     .range
  //     .start
  //     .set(Instant::now())
  //     .expect("should only call once");
  // }

  // pub fn mark_restoring_end(&self) {
  //   self
  //     .restoring
  //     .range
  //     .end
  //     .set(Instant::now())
  //     .expect("should only call once");
  // }

  pub fn mark_integration_start(&self) {
    self
      .integration
      .range
      .start
      .set(Instant::now())
      .expect("should only call once");
  }

  pub fn mark_integration_end(&self) {
    self
      .integration
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

  // TODO: store module to cache is not implemented yet
  // pub fn mark_storing_start(&self) {
  //   self
  //     .storing
  //     .range
  //     .start
  //     .set(Instant::now())
  //     .expect("should only call once");
  // }

  // pub fn mark_storing_end(&self) {
  //   self
  //     .storing
  //     .range
  //     .end
  //     .set(Instant::now())
  //     .expect("should only call once");
  // }

  // pub fn merge(&mut self, other: Self) {
  //   self.additional_factories += other.factory.duration().expect("should have duration");
  //   self.additional_factory_times.push(TimeRange::with_value(
  //     *other
  //       .factory
  //       .range
  //       .start
  //       .get()
  //       .expect("should have duration"),
  //     *other.factory.range.end.get().expect("should have duration"),
  //   ));
  // }
}
