use rspack_sources::{encode_mappings, DecodableMap, Mapping, OriginalLocation};
use sourcemap::SourceMap;

#[derive(Debug)]
pub(crate) struct SwcSourceMapWrapper(SourceMap);

impl SwcSourceMapWrapper {
  pub fn new(map: SourceMap) -> Self {
    Self(map)
  }
}

impl DecodableMap for SwcSourceMapWrapper {
  fn file(&self) -> Option<&str> {
    self.0.get_file()
  }

  fn mappings(&self) -> String {
    encode_mappings(self.0.tokens().map(|token| Mapping {
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

  fn decoded_mappings(&self) -> Vec<Mapping> {
    self
      .0
      .tokens()
      .map(|token| Mapping {
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
      })
      .collect::<Vec<_>>()
  }

  fn sources<'a>(&'a self) -> Box<dyn Iterator<Item = &'a str> + 'a> {
    Box::new(self.0.sources())
  }

  fn sources_content<'a>(&'a self) -> Box<dyn Iterator<Item = &'a str> + 'a> {
    Box::new(self.0.source_contents().flatten())
  }

  fn names<'a>(&'a self) -> Box<dyn Iterator<Item = &'a str> + 'a> {
    Box::new(self.0.names())
  }

  fn source_root(&self) -> Option<&str> {
    self.0.get_source_root()
  }

  fn to_json(self: Box<Self>) -> rspack_sources::Result<String> {
    let mut buf = vec![];
    self
      .0
      .to_writer(&mut buf)
      .unwrap_or_else(|e| panic!("{}", e.to_string()));
    Ok(unsafe { String::from_utf8_unchecked(buf) })
  }

  fn source(&self, index: usize) -> Option<&str> {
    self.0.get_source(index as u32)
  }

  fn source_content(&self, index: usize) -> Option<&str> {
    self.0.get_source_contents(index as u32)
  }

  fn name(&self, index: usize) -> Option<&str> {
    self.0.get_name(index as u32)
  }
}
