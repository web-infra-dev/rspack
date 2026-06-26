use std::borrow::Cow;

use lightningcss::printer::{
  OriginalLocation as LightningOriginalLocation, SourceMap as LightningSourceMap,
};
use rspack_core::rspack_sources::{
  Mapping, OriginalLocation as RspackOriginalLocation, SourceMap, encode_mappings,
};

#[derive(Default)]
pub struct RspackSourceMap {
  sources: Vec<Cow<'static, str>>,
  sources_content: Vec<Cow<'static, str>>,
  names: Vec<Cow<'static, str>>,
  mappings: Vec<Mapping>,
  source_root: Option<Cow<'static, str>>,
}

impl RspackSourceMap {
  pub fn with_source_root(source_root: Option<&str>) -> Self {
    Self {
      source_root: source_root.map(|source_root| Cow::Owned(source_root.to_string())),
      ..Default::default()
    }
  }

  pub fn finish(self) -> SourceMap<'static> {
    let mut source_map = SourceMap::new(
      encode_mappings(self.mappings.into_iter()),
      self.sources,
      self.sources_content,
      self.names,
    );
    source_map.set_source_root(self.source_root);
    source_map
  }
}

impl LightningSourceMap for RspackSourceMap {
  fn add_source(&mut self, source: &str) -> u32 {
    if let Some(index) = self.sources.iter().position(|s| s.as_ref() == source) {
      index as u32
    } else {
      self.sources.push(Cow::Owned(source.to_string()));
      (self.sources.len() - 1) as u32
    }
  }

  fn add_name(&mut self, name: &str) -> u32 {
    if let Some(index) = self.names.iter().position(|n| n.as_ref() == name) {
      index as u32
    } else {
      self.names.push(Cow::Owned(name.to_string()));
      (self.names.len() - 1) as u32
    }
  }

  fn set_source_content(&mut self, source_index: u32, source_content: &str) {
    let source_index = source_index as usize;
    if self.sources_content.len() <= source_index {
      self
        .sources_content
        .resize_with(source_index + 1, || Cow::Borrowed(""));
    }
    self.sources_content[source_index] = Cow::Owned(source_content.to_string());
  }

  fn add_mapping(
    &mut self,
    generated_line: u32,
    generated_column: u32,
    original: Option<LightningOriginalLocation>,
  ) {
    self.mappings.push(Mapping {
      generated_line: generated_line + 1,
      generated_column,
      original: original.map(|original| RspackOriginalLocation {
        source_index: original.source,
        original_line: original.original_line + 1,
        original_column: original.original_column,
        name_index: original.name,
      }),
    });
  }

  fn from_data_url(_source_root: &str, _data_url: &str) -> Option<Self> {
    None
  }

  fn find_closest_mapping(
    &mut self,
    _line: u32,
    _column: u32,
  ) -> Option<LightningOriginalLocation> {
    None
  }

  fn get_source(&self, source_index: u32) -> Option<&str> {
    self
      .sources
      .get(source_index as usize)
      .map(|source| source.as_ref())
  }

  fn get_name(&self, name_index: u32) -> Option<&str> {
    self
      .names
      .get(name_index as usize)
      .map(|name| name.as_ref())
  }

  fn get_source_content(&self, source_index: u32) -> Option<&str> {
    self
      .sources_content
      .get(source_index as usize)
      .map(|source_content| source_content.as_ref())
  }
}
