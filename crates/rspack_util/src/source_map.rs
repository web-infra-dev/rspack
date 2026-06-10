use bitflags::bitflags;
use rspack_cacheable::cacheable;

#[cacheable]
#[derive(Debug, PartialEq, Hash, Eq, Clone, Copy, PartialOrd, Ord)]
pub struct SourceMapKind(u8);

bitflags! {
    impl SourceMapKind: u8 {
      const SourceMap = 1 << 0;
      const SimpleSourceMap = 1 << 1;
      const Cheap = 1 << 2;
      const NoSources = 1 << 3;
      const Inline = 1 << 4;
  }
}

impl Default for SourceMapKind {
  fn default() -> Self {
    SourceMapKind::empty()
  }
}

impl SourceMapKind {
  pub fn from_enabled(enabled: bool) -> Self {
    if enabled {
      SourceMapKind::SourceMap
    } else {
      SourceMapKind::empty()
    }
  }

  pub fn from_module(module: bool) -> Self {
    if module {
      SourceMapKind::SourceMap
    } else {
      SourceMapKind::SimpleSourceMap
    }
  }

  pub fn with_cheap(mut self, cheap: bool) -> Self {
    if cheap {
      self |= SourceMapKind::Cheap;
    }
    self
  }

  pub fn with_no_sources(mut self, no_sources: bool) -> Self {
    if no_sources {
      self |= SourceMapKind::NoSources;
    }
    self
  }

  pub fn with_sources_content(self, sources_content: bool) -> Self {
    self.with_no_sources(!sources_content)
  }

  pub fn with_inline(mut self, inline: bool) -> Self {
    if inline {
      self |= SourceMapKind::Inline;
    }
    self
  }

  pub fn enabled(&self) -> bool {
    self.source_map() || self.simple_source_map()
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

  pub fn no_sources(&self) -> bool {
    self.contains(SourceMapKind::NoSources)
  }

  pub fn inline(&self) -> bool {
    self.contains(SourceMapKind::Inline)
  }

  pub fn inline_sources_content(&self) -> bool {
    self.source_map() && !self.no_sources()
  }

  pub fn emit_columns(&self) -> bool {
    !self.cheap()
  }
}

pub trait ModuleSourceMapConfig {
  fn get_source_map_kind(&self) -> &SourceMapKind;
  fn set_source_map_kind(&mut self, source_map: SourceMapKind);
}
