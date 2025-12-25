use std::time::Instant;

use rspack_cacheable::{
  ContextGuard, Result, cacheable,
  with::{Custom, CustomConverter},
};

#[cacheable(with=Custom)]
#[derive(Debug, Default, Clone)]
enum ProfileState {
  #[default]
  Pending,
  Started(Instant),
  // u64 is enough to store the time consumption
  Finish(u64),
}

impl CustomConverter for ProfileState {
  type Target = Option<u64>;
  fn serialize(&self, _guard: &ContextGuard) -> Result<Self::Target> {
    Ok(self.duration())
  }
  fn deserialize(data: Self::Target, _guard: &ContextGuard) -> Result<Self> {
    if let Some(time) = data {
      Ok(ProfileState::Finish(time))
    } else {
      Ok(ProfileState::default())
    }
  }
}

impl ProfileState {
  fn start(&mut self) {
    *self = Self::Started(Instant::now())
  }

  fn end(&mut self) {
    match self {
      Self::Started(i) => {
        let time = Instant::now().duration_since(*i);
        *self = Self::Finish(time.as_millis() as u64)
      }
      _ => panic!("Unable to end an unstarted profiler"),
    }
  }

  fn duration(&self) -> Option<u64> {
    match self {
      Self::Finish(time) => Some(*time),
      _ => None,
    }
  }
}

// https://github.com/webpack/webpack/blob/4809421990a20dfefa06e6445191e65001e75f88/lib/ModuleProfile.js
// NOTE: Rspack has different cache design, remove cache related profiles

#[cacheable]
#[derive(Debug, Default, Clone)]
pub struct ModuleProfile {
  factory: ProfileState,
  building: ProfileState,
}

impl ModuleProfile {
  pub fn mark_factory_start(&mut self) {
    self.factory.start();
  }

  pub fn mark_factory_end(&mut self) {
    self.factory.end();
  }

  pub fn mark_building_start(&mut self) {
    self.building.start();
  }

  pub fn mark_building_end(&mut self) {
    self.building.end();
  }

  pub fn factory_duration(&self) -> Option<u64> {
    self.factory.duration()
  }

  pub fn building_duration(&self) -> Option<u64> {
    self.building.duration()
  }
}
