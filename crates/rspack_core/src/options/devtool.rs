use bitflags::bitflags;

bitflags! {
  struct DevtoolFlags: u8 {
    const SOURCE_MAP = 0b00000001;
    const CHEAP = 0b00000010;
    const INLINE = 0b00000100;
    const NO_SOURCES = 0b00001000;
    const HIDDEN = 0b00010000;
    const MODULE = 0b00100000;
    // TODO: eval source map for better development (re)builds performance.
    const EVAL = 0b01000000;
  }
}

#[derive(Debug, Hash)]
pub struct Devtool {
  inner: DevtoolFlags,
}

impl Default for Devtool {
  fn default() -> Self {
    Self {
      inner: DevtoolFlags::empty(),
    }
  }
}

impl Devtool {
  pub fn add_source_map(&mut self) {
    self.inner.insert(DevtoolFlags::SOURCE_MAP)
  }
  pub fn add_cheap(&mut self) {
    self.inner.insert(DevtoolFlags::CHEAP)
  }
  pub fn add_inline(&mut self) {
    self.inner.insert(DevtoolFlags::INLINE)
  }
  pub fn add_no_sources(&mut self) {
    self.inner.insert(DevtoolFlags::NO_SOURCES)
  }
  pub fn add_hidden(&mut self) {
    self.inner.insert(DevtoolFlags::HIDDEN)
  }
  pub fn add_module(&mut self) {
    self.inner.insert(DevtoolFlags::MODULE)
  }
  pub fn add_eval(&mut self) {
    self.inner.insert(DevtoolFlags::EVAL)
  }

  pub fn source_map(&self) -> bool {
    self.inner.contains(DevtoolFlags::SOURCE_MAP)
  }
  pub fn cheap(&self) -> bool {
    self.inner.contains(DevtoolFlags::CHEAP)
  }
  pub fn inline(&self) -> bool {
    self.inner.contains(DevtoolFlags::INLINE)
  }
  pub fn no_sources(&self) -> bool {
    self.inner.contains(DevtoolFlags::NO_SOURCES)
  }
  pub fn hidden(&self) -> bool {
    self.inner.contains(DevtoolFlags::HIDDEN)
  }
  pub fn module(&self) -> bool {
    self.inner.contains(DevtoolFlags::MODULE)
  }
  pub fn eval(&self) -> bool {
    self.inner.contains(DevtoolFlags::EVAL)
  }

  /// for loader author whether need to care about source map.
  pub fn enabled(&self) -> bool {
    self.source_map() && self.module()
  }
}

impl From<String> for Devtool {
  fn from(s: String) -> Self {
    let mut devtool = Self::default();
    if s.contains("source-map") {
      devtool.add_source_map();
    }
    if s.contains("inline") {
      devtool.add_inline();
    }
    if s.contains("nosources") {
      devtool.add_no_sources();
    }
    if s.contains("hidden") {
      devtool.add_hidden();
    }
    if s.contains("cheap") {
      devtool.add_cheap();
    }
    if s.contains("module") || !s.contains("cheap") {
      devtool.add_module();
    }
    if s.contains("eval") {
      devtool.add_eval();
    }
    devtool
  }
}
