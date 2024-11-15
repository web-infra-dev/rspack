use once_cell::sync::OnceCell;
use rspack_sources::{encode_mappings, DecodableMap, Mapping, OriginalLocation};
use sourcemap::SourceMap;

#[derive(Debug)]
pub(crate) struct RspackSourceMap {
  inner: SourceMap,
  mappings: OnceCell<String>,
}

impl RspackSourceMap {
  pub fn new(map: SourceMap) -> Self {
    Self {
      inner: map,
      mappings: Default::default(),
    }
  }
}

impl DecodableMap for RspackSourceMap {
  fn file(&self) -> Option<&str> {
    self.inner.get_file()
  }

  fn mappings(&self) -> &str {
    self
      .mappings
      .get_or_init(|| encode_mappings(self.decoded_mappings()))
  }

  fn decoded_mappings<'a>(&'a self) -> Box<dyn Iterator<Item = Mapping> + 'a> {
    Box::new(self.inner.tokens().map(|token| Mapping {
      generated_line: token.get_dst_line() + 1,
      generated_column: token.get_dst_col(),
      original: if token.has_source() {
        Some(OriginalLocation {
          source_index: token.get_src_id(),
          original_line: token.get_src_line() + 1,
          original_column: token.get_src_col(),
          name_index: if token.has_name() {
            Some(token.get_name_id())
          } else {
            None
          },
        })
      } else {
        None
      },
    }))
  }

  fn sources<'a>(&'a self) -> Box<dyn Iterator<Item = &'a str> + 'a> {
    Box::new(self.inner.sources())
  }

  fn sources_content<'a>(&'a self) -> Box<dyn Iterator<Item = &'a str> + 'a> {
    Box::new(self.inner.source_contents().flatten())
  }

  fn names<'a>(&'a self) -> Box<dyn Iterator<Item = &'a str> + 'a> {
    Box::new(self.inner.names())
  }

  fn source_root(&self) -> Option<&str> {
    self.inner.get_source_root()
  }

  fn to_json(self: Box<Self>) -> rspack_sources::Result<String> {
    let mut buf = vec![];
    self
      .inner
      .to_writer(&mut buf)
      .unwrap_or_else(|e| panic!("{}", e.to_string()));
    Ok(unsafe { String::from_utf8_unchecked(buf) })
  }

  fn source(&self, index: usize) -> Option<&str> {
    self.inner.get_source(index as u32)
  }

  fn source_content(&self, index: usize) -> Option<&str> {
    self.inner.get_source_contents(index as u32)
  }

  fn name(&self, index: usize) -> Option<&str> {
    self.inner.get_name(index as u32)
  }
}
