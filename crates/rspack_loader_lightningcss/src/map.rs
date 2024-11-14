use once_cell::sync::OnceCell;
use parcel_sourcemap::SourceMap;
use rspack_sources::{DecodableSourceMap, Mapping, OriginalLocation};

#[derive(Debug)]
pub(crate) struct RspackSourceMap {
  inner: SourceMap,
  mappings: OnceCell<Vec<Mapping>>,
}

impl RspackSourceMap {
  pub fn new(map: SourceMap) -> Self {
    Self {
      inner: map,
      mappings: Default::default(),
    }
  }
}

impl DecodableSourceMap for RspackSourceMap {
  fn file(&self) -> Option<&str> {
    None
  }

  fn mappings(&self) -> &str {
    unimplemented!()
  }

  fn decoded_mappings<'a>(&'a self) -> Box<dyn Iterator<Item = Mapping> + 'a> {
    Box::new(
      self
        .mappings
        .get_or_init(|| {
          self
            .inner
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
        })
        .iter()
        .cloned(),
    )
  }

  fn sources<'a>(&'a self) -> Box<dyn Iterator<Item = &'a str> + 'a> {
    Box::new(
      self
        .inner
        .get_sources()
        .iter()
        .map(|source| source.as_str()),
    )
  }

  fn sources_content<'a>(&'a self) -> Box<dyn Iterator<Item = &'a str> + 'a> {
    Box::new(
      self
        .inner
        .get_sources_content()
        .iter()
        .map(|source| source.as_str()),
    )
  }

  fn names<'a>(&'a self) -> Box<dyn Iterator<Item = &'a str> + 'a> {
    Box::new(self.inner.get_names().iter().map(|source| source.as_str()))
  }

  fn source_root(&self) -> Option<&str> {
    Some(&self.inner.project_root)
  }

  fn to_json(mut self: Box<Self>) -> rspack_sources::Result<String> {
    Ok(
      self
        .inner
        .to_json(None)
        .unwrap_or_else(|e| panic!("{}", e.to_string())),
    )
  }
}
