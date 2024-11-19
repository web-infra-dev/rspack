use parcel_sourcemap::SourceMap;
use rspack_sources::{encode_mappings, DecodableMap, Mapping, OriginalLocation};

#[derive(Debug)]
pub(crate) struct LightningcssSourceMapWrapper(SourceMap);

impl LightningcssSourceMapWrapper {
  pub fn new(map: SourceMap) -> Self {
    Self(map)
  }
}

impl DecodableMap for LightningcssSourceMapWrapper {
  fn file(&self) -> Option<&str> {
    None
  }

  fn mappings(&self) -> String {
    encode_mappings(self.0.get_mappings().iter().map(|mapping| Mapping {
      generated_line: mapping.generated_line,
      generated_column: mapping.generated_column,
      original: mapping.original.map(|original| OriginalLocation {
        source_index: original.source,
        original_line: original.original_line,
        original_column: original.original_column,
        name_index: original.name,
      }),
    }))
  }

  fn decoded_mappings(&self) -> Vec<Mapping> {
    self
      .0
      .get_mappings()
      .iter()
      .map(|mapping| Mapping {
        generated_line: mapping.generated_line,
        generated_column: mapping.generated_column,
        original: mapping.original.map(|original| OriginalLocation {
          source_index: original.source,
          original_line: original.original_line,
          original_column: original.original_column,
          name_index: original.name,
        }),
      })
      .collect::<Vec<_>>()
  }

  fn sources<'a>(&'a self) -> Box<dyn Iterator<Item = &'a str> + 'a> {
    Box::new(self.0.get_sources().iter().map(|source| source.as_str()))
  }

  fn sources_content<'a>(&'a self) -> Box<dyn Iterator<Item = &'a str> + 'a> {
    Box::new(
      self
        .0
        .get_sources_content()
        .iter()
        .map(|source| source.as_str()),
    )
  }

  fn names<'a>(&'a self) -> Box<dyn Iterator<Item = &'a str> + 'a> {
    Box::new(self.0.get_names().iter().map(|source| source.as_str()))
  }

  fn source_root(&self) -> Option<&str> {
    Some(&self.0.project_root)
  }

  fn to_json(mut self: Box<Self>) -> rspack_sources::Result<String> {
    Ok(
      self
        .0
        .to_json(None)
        .unwrap_or_else(|e| panic!("{}", e.to_string())),
    )
  }

  fn source(&self, index: usize) -> Option<&str> {
    self.0.get_source(index as u32).ok()
  }

  fn source_content(&self, index: usize) -> Option<&str> {
    self.0.get_source_content(index as u32).ok()
  }

  fn name(&self, index: usize) -> Option<&str> {
    self.0.get_name(index as u32).ok()
  }
}
