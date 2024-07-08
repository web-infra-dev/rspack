use bitflags::bitflags;

bitflags! {
  #[derive(Debug, PartialEq, Eq, Clone, Copy)]
  pub struct SourceMapKind: u8 {
      const SourceMap = 1 << 0;
      const SimpleSourceMap = 1 << 1;
      const Cheap = 1 << 2;
  }
}

impl Default for SourceMapKind {
  fn default() -> Self {
    SourceMapKind::empty()
  }
}

impl SourceMapKind {
  pub fn enabled(&self) -> bool {
    !self.is_empty()
  }

  pub fn source_map(&self) -> bool {
    self.contains(SourceMapKind::SourceMap)
  }

  pub fn simple_source_map(&self) -> bool {
    self.contains(SourceMapKind::SimpleSourceMap)
  }

  pub fn cheap(&self) -> bool {
    self.contains(SourceMapKind::Cheap)
  }
}

pub trait ModuleSourceMapConfig {
  fn get_source_map_kind(&self) -> &SourceMapKind;
  fn set_source_map_kind(&mut self, source_map: SourceMapKind);
}
