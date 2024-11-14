use rspack_sources::{DecodableSourceMap, Mapping, OriginalLocation};
use sourcemap::SourceMap;

#[derive(Debug)]
pub(crate) struct RspackSourceMap {
  inner: SourceMap,
}

impl RspackSourceMap {
  pub fn new(map: SourceMap) -> Self {
    Self { inner: map }
  }
}

impl DecodableSourceMap for RspackSourceMap {
  fn file(&self) -> Option<&str> {
    self.inner.get_file()
  }

  fn mappings(&self) -> &str {
    unimplemented!()
  }

  fn decoded_mappings<'a>(&'a self) -> Box<dyn Iterator<Item = Mapping> + 'a> {
    let iter = self.inner.tokens().into_iter().map(|token| Mapping {
      generated_line: token.get_dst_line(),
      generated_column: token.get_dst_col(),
      original: if token.has_source() {
        Some(OriginalLocation {
          source_index: token.get_src_id(),
          original_line: token.get_src_line(),
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
    });
    Box::new(iter)
  }

  fn sources<'a>(&'a self) -> Box<dyn Iterator<Item = &'a str> + 'a> {
    Box::new(self.inner.sources())
  }

  fn sources_content<'a>(&'a self) -> Box<dyn Iterator<Item = &'a str> + 'a> {
    Box::new(self.inner.source_contents().filter_map(|content| content))
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
}
