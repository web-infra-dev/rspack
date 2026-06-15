use std::{
  borrow::Cow,
  hash::{Hash, Hasher},
  sync::Arc,
};

use crate::{
  MapOptions, Source, SourceMap, SourceValue,
  helpers::{
    Chunks, GeneratedInfo, StreamChunks, TextSpan, get_generated_source_info, get_map,
    split_into_lines, split_into_potential_tokens,
  },
  object_pool::ObjectPool,
  source::{Mapping, OriginalLocation},
};

/// Represents source code, it will create source map for the source code,
/// but the source map is created by splitting the source code at typical
/// statement borders (`;`, `{`, `}`).
///
/// - [webpack-sources docs](https://github.com/webpack/webpack-sources/#originalsource).
///
/// ```
/// use rspack_sources::{OriginalSource, MapOptions, Source, SourceExt, ObjectPool};
///
/// let input = "if (hello()) { world(); hi(); there(); } done();\nif (hello()) { world(); hi(); there(); } done();";
/// let source = OriginalSource::new(input, "file.js");
/// assert_eq!(source.source().into_string_lossy(), input);
/// assert_eq!(
///   source.map(&ObjectPool::default(), &MapOptions::default()).unwrap().mappings(),
///   "AAAA,eAAe,SAAS,MAAM,WAAW;AACzC,eAAe,SAAS,MAAM,WAAW",
/// );
/// assert_eq!(
///   source.map(&ObjectPool::default(), &MapOptions::new(false)).unwrap().mappings(),
///   "AAAA;AACA",
/// );
/// ```
#[derive(Clone, Eq)]
pub struct OriginalSource {
  value: Box<str>,
  name: Box<str>,
}

impl OriginalSource {
  /// Create a [OriginalSource].
  pub fn new(value: impl Into<Box<str>>, name: impl Into<Box<str>>) -> Self {
    Self {
      value: value.into(),
      name: name.into(),
    }
  }

  /// Get the name of the source file.
  pub fn name(&self) -> &str {
    &self.name
  }

  /// Get the value as a shared string reference.
  pub fn value(&self) -> &str {
    &self.value
  }
}

impl Source for OriginalSource {
  fn source(&self) -> SourceValue<'_> {
    SourceValue::String(Cow::Borrowed(&self.value))
  }

  fn rope<'a>(&'a self, on_chunk: &mut dyn FnMut(&'a str)) {
    on_chunk(self.value.as_ref())
  }

  fn buffer(&self) -> Cow<'_, [u8]> {
    Cow::Borrowed(self.value.as_bytes())
  }

  fn size(&self) -> usize {
    self.value.len()
  }

  fn map<'a>(&'a self, object_pool: &ObjectPool, options: &MapOptions) -> Option<SourceMap<'a>> {
    let chunks = self.stream_chunks();
    get_map(object_pool, chunks.as_ref(), options).map(SourceMap::from_fields)
  }

  fn map_static(
    self: Arc<Self>,
    object_pool: &ObjectPool,
    options: &MapOptions,
  ) -> Option<SourceMap<'static>> {
    let owner = self.clone();
    self
      .as_ref()
      .map(object_pool, options)
      .map(|map| map.into_static(owner))
  }

  fn to_writer(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
    writer.write_all(self.value.as_bytes())
  }
}

impl Hash for OriginalSource {
  fn hash<H: Hasher>(&self, state: &mut H) {
    "OriginalSource".hash(state);
    self.buffer().hash(state);
    self.name.hash(state);
  }
}

impl PartialEq for OriginalSource {
  fn eq(&self, other: &Self) -> bool {
    self.value == other.value && self.name == other.name
  }
}

impl std::fmt::Debug for OriginalSource {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
    let indent = f.width().unwrap_or(0);
    let indent_str = format!("{:indent$}", "", indent = indent);

    writeln!(f, "{indent_str}OriginalSource::new(")?;
    writeln!(f, "{indent_str}  {:?},", self.value)?;
    writeln!(f, "{indent_str}  {:?},", self.name)?;
    write!(f, "{indent_str}).boxed()")
  }
}

struct OriginalSourceChunks<'a>(&'a OriginalSource);

impl<'source> OriginalSourceChunks<'source> {
  pub fn new(source: &'source OriginalSource) -> Self {
    Self(source)
  }
}

impl<'source> Chunks<'source> for OriginalSourceChunks<'source> {
  fn stream<'chunk>(
    &'chunk self,
    _object_pool: &ObjectPool,
    options: &MapOptions,
    on_chunk: crate::helpers::OnChunk<'_, 'chunk>,
    on_source: crate::helpers::OnSource<'_, 'source>,
    _on_name: crate::helpers::OnName<'_, 'source>,
  ) -> GeneratedInfo {
    on_source(0, Cow::Borrowed(&self.0.name), Some(self.0.value.as_ref()));
    let source = TextSpan::new(self.0.value.as_ref());
    if options.columns {
      // With column info we need to read all lines and split them
      let mut line = 1;
      let mut column = 0;
      for token in split_into_potential_tokens(self.0.value.as_ref()) {
        let is_end_of_line = token.ends_with("\n");
        if is_end_of_line && token.len() == 1 {
          if !options.final_source {
            on_chunk(
              Some(source.subspan(token)),
              Mapping {
                generated_line: line,
                generated_column: column,
                original: None,
              },
            );
          }
        } else {
          on_chunk(
            (!options.final_source).then_some(source.subspan(token)),
            Mapping {
              generated_line: line,
              generated_column: column,
              original: Some(OriginalLocation {
                source_index: 0,
                original_line: line,
                original_column: column,
                name_index: None,
              }),
            },
          );
        }
        if is_end_of_line {
          line += 1;
          column = 0;
        } else {
          column += source.utf16_len_of(token) as u32;
        }
      }
      GeneratedInfo {
        generated_line: line,
        generated_column: column,
      }
    } else if options.final_source {
      // Without column info and with final source we only
      // need meta info to generate mapping
      let result = get_generated_source_info(source);
      if result.generated_column == 0 {
        for line in 1..result.generated_line {
          on_chunk(
            None,
            Mapping {
              generated_line: line,
              generated_column: 0,
              original: Some(OriginalLocation {
                source_index: 0,
                original_line: line,
                original_column: 0,
                name_index: None,
              }),
            },
          );
        }
      } else {
        for line in 1..=result.generated_line {
          on_chunk(
            None,
            Mapping {
              generated_line: line,
              generated_column: 0,
              original: Some(OriginalLocation {
                source_index: 0,
                original_line: line,
                original_column: 0,
                name_index: None,
              }),
            },
          );
        }
      }
      result
    } else {
      // Without column info, but also without final source
      // we need to split source by lines
      let mut line = 1;
      let mut last_line = None;
      for l in split_into_lines(self.0.value.as_ref()) {
        on_chunk(
          (!options.final_source).then_some(source.subspan(l)),
          Mapping {
            generated_line: line,
            generated_column: 0,
            original: Some(OriginalLocation {
              source_index: 0,
              original_line: line,
              original_column: 0,
              name_index: None,
            }),
          },
        );
        line += 1;
        last_line = Some(l);
      }
      if let Some(last_line) = last_line.filter(|last_line| !last_line.ends_with('\n')) {
        GeneratedInfo {
          generated_line: line - 1,
          generated_column: source.utf16_len_of(last_line) as u32,
        }
      } else {
        GeneratedInfo {
          generated_line: line,
          generated_column: 0,
        }
      }
    }
  }
}

impl StreamChunks for OriginalSource {
  fn stream_chunks<'a>(&'a self) -> Box<dyn Chunks<'a> + 'a> {
    Box::new(OriginalSourceChunks::new(self))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{ConcatSource, ReplaceSource, SourceExt};

  #[test]
  fn should_handle_multiline_string() {
    let source = OriginalSource::new("Line1\n\nLine3\n", "file.js");
    let result_text = source.source();
    let result_map = source
      .map(&ObjectPool::default(), &MapOptions::default())
      .unwrap();
    let result_list_map = source
      .map(&ObjectPool::default(), &MapOptions::new(false))
      .unwrap();

    assert_eq!(result_text.into_string_lossy(), "Line1\n\nLine3\n");
    assert_eq!(result_map.get_source(0), Some("file.js"));
    assert_eq!(result_list_map.get_source(0), Some("file.js"));
    assert_eq!(
      result_map.get_source_content(0).map(AsRef::as_ref),
      Some("Line1\n\nLine3\n")
    );
    assert_eq!(
      result_list_map.get_source_content(0).map(AsRef::as_ref),
      Some("Line1\n\nLine3\n")
    );
    assert_eq!(result_map.mappings(), "AAAA;;AAEA");
    assert_eq!(result_list_map.mappings(), "AAAA;AACA;AACA");
  }

  #[test]
  fn should_handle_empty_string() {
    let source = OriginalSource::new("", "file.js");
    let result_text = source.source();
    let result_map = source.map(&ObjectPool::default(), &MapOptions::default());
    let result_list_map = source.map(&ObjectPool::default(), &MapOptions::new(false));

    assert_eq!(result_text.into_string_lossy(), "");
    assert!(result_map.is_none());
    assert!(result_list_map.is_none());
  }

  #[test]
  fn should_omit_mappings_for_columns_with_node() {
    let source = OriginalSource::new("Line1\n\nLine3\n", "file.js");
    let result_map = source
      .map(&ObjectPool::default(), &MapOptions::new(false))
      .unwrap();
    assert_eq!(result_map.mappings(), "AAAA;AACA;AACA");
  }

  #[test]
  fn should_return_the_correct_size_for_binary_files() {
    let source = OriginalSource::new(String::from_utf8(vec![0; 256]).unwrap(), "file.wasm");
    assert_eq!(source.size(), 256);
  }

  #[test]
  fn should_return_the_correct_size_for_unicode_files() {
    let source = OriginalSource::new("😋", "file.js");
    assert_eq!(source.size(), 4);
  }

  #[test]
  fn should_split_code_into_statements() {
    let input = "if (hello()) { world(); hi(); there(); } done();\nif (hello()) { world(); hi(); there(); } done();";
    let source = OriginalSource::new(input, "file.js");
    assert_eq!(source.source().into_string_lossy(), input);
    assert_eq!(
      source
        .map(&ObjectPool::default(), &MapOptions::default())
        .unwrap()
        .mappings(),
      "AAAA,eAAe,SAAS,MAAM,WAAW;AACzC,eAAe,SAAS,MAAM,WAAW",
    );
    assert_eq!(
      source
        .map(&ObjectPool::default(), &MapOptions::new(false))
        .unwrap()
        .mappings(),
      "AAAA;AACA",
    );
  }

  // Fix https://github.com/web-infra-dev/rspack/issues/6793
  #[test]
  fn fix_rspack_issue_6793() {
    let code1 = "hello\n\n";
    let source1 = OriginalSource::new(code1, "test.txt");
    let source1 = ReplaceSource::new(source1);

    let code2 = "world";
    let source2 = OriginalSource::new(code2, "world.txt");

    let concat = ConcatSource::new([source1.boxed(), source2.boxed()]);
    let map = concat
      .map(&ObjectPool::default(), &MapOptions::new(false))
      .unwrap();
    assert_eq!(map.mappings(), "AAAA;AACA;ACDA",);
  }

  #[test]
  fn test_potential_tokens_multi_unit_utf16() {
    let code = "var i18n = JSON.parse('{\"魑魅魍魉\":{\"en-US\":\"Evil spirits\",\"zh-CN\":\"魑魅魍魉\"}}');\nvar __webpack_exports___ = i18n[\"魑魅魍魉\"];\nexport { __webpack_exports___ as 魑魅魍魉 };";
    let source = OriginalSource::new(code, "test.js");
    let mut chunks = vec![];
    let object_pool = ObjectPool::default();
    let handle = source.stream_chunks();
    let generated_info = handle.stream(
      &object_pool,
      &MapOptions::default(),
      &mut |chunk, mapping| {
        chunks.push((chunk.unwrap().as_str(), mapping));
      },
      &mut |_source_index, _source, _source_content| {},
      &mut |_name_index, _name| {},
    );

    assert_eq!(
      generated_info,
      GeneratedInfo {
        generated_line: 3,
        generated_column: 40
      }
    );
    assert_eq!(
      chunks,
      vec![
        (
          "var i18n = JSON.parse('{",
          Mapping {
            generated_line: 1,
            generated_column: 0,
            original: Some(OriginalLocation {
              source_index: 0,
              original_line: 1,
              original_column: 0,
              name_index: None
            })
          }
        ),
        (
          "\"魑魅魍魉\":{",
          Mapping {
            generated_line: 1,
            generated_column: 24,
            original: Some(OriginalLocation {
              source_index: 0,
              original_line: 1,
              original_column: 24,
              name_index: None
            })
          }
        ),
        (
          "\"en-US\":\"Evil spirits\",\"zh-CN\":\"魑魅魍魉\"}}",
          Mapping {
            generated_line: 1,
            generated_column: 32,
            original: Some(OriginalLocation {
              source_index: 0,
              original_line: 1,
              original_column: 32,
              name_index: None
            })
          }
        ),
        (
          "');\n",
          Mapping {
            generated_line: 1,
            generated_column: 71,
            original: Some(OriginalLocation {
              source_index: 0,
              original_line: 1,
              original_column: 71,
              name_index: None
            })
          }
        ),
        (
          "var __webpack_exports___ = i18n[\"魑魅魍魉\"];\n",
          Mapping {
            generated_line: 2,
            generated_column: 0,
            original: Some(OriginalLocation {
              source_index: 0,
              original_line: 2,
              original_column: 0,
              name_index: None
            })
          }
        ),
        (
          "export { ",
          Mapping {
            generated_line: 3,
            generated_column: 0,
            original: Some(OriginalLocation {
              source_index: 0,
              original_line: 3,
              original_column: 0,
              name_index: None
            })
          }
        ),
        (
          "__webpack_exports___ as 魑魅魍魉 };",
          Mapping {
            generated_line: 3,
            generated_column: 9,
            original: Some(OriginalLocation {
              source_index: 0,
              original_line: 3,
              original_column: 9,
              name_index: None
            })
          }
        )
      ]
    )
  }
}
