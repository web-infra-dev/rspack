use std::{
  borrow::Cow,
  cell::RefCell,
  hash::{Hash, Hasher},
  sync::{Mutex, OnceLock},
};

use rustc_hash::FxHashMap as HashMap;

use crate::{
  BoxSource, MapOptions, RawStringSource, Source, SourceExt, SourceMap, SourceValue,
  helpers::{Chunks, GeneratedInfo, StreamChunks, get_map},
  linear_map::LinearMap,
  object_pool::ObjectPool,
  source::{Mapping, OriginalLocation},
};

/// Concatenate multiple [Source]s to a single [Source].
///
/// - [webpack-sources docs](https://github.com/webpack/webpack-sources/#concatsource).
///
/// ```
/// use rspack_sources::{
///   BoxSource, ConcatSource, MapOptions, ObjectPool, OriginalSource, RawStringSource, Source,
///   SourceExt, SourceMap,
/// };
///
/// let mut source = ConcatSource::new([
///   RawStringSource::from("Hello World\n".to_string()).boxed(),
///   OriginalSource::new(
///     "console.log('test');\nconsole.log('test2');\n",
///     "console.js",
///   )
///   .boxed(),
/// ]);
/// source.add(OriginalSource::new("Hello2\n", "hello.md"));
///
/// assert_eq!(source.size(), 62);
/// assert_eq!(
///   source.source().into_string_lossy(),
///   "Hello World\nconsole.log('test');\nconsole.log('test2');\nHello2\n"
/// );
/// assert_eq!(
///   source
///     .map(&ObjectPool::default(), &MapOptions::new(false))
///     .unwrap(),
///   SourceMap::from_json(
///     r#"{
///       "version": 3,
///       "mappings": ";AAAA;AACA;ACDA",
///       "names": [],
///       "sources": ["console.js", "hello.md"],
///       "sourcesContent": [
///         "console.log('test');\nconsole.log('test2');\n",
///         "Hello2\n"
///       ]
///     }"#,
///   )
///   .unwrap()
/// );
/// ```
#[derive(Default)]
pub struct ConcatSource {
  children: Mutex<Vec<BoxSource>>,
  is_optimized: OnceLock<Vec<BoxSource>>,
}

impl Clone for ConcatSource {
  fn clone(&self) -> Self {
    Self {
      children: Mutex::new(self.children.lock().unwrap().clone()),
      is_optimized: match self.is_optimized.get() {
        Some(children) => {
          let once_lock = OnceLock::new();
          once_lock.get_or_init(|| children.clone());
          once_lock
        }
        None => OnceLock::default(),
      },
    }
  }
}

impl std::fmt::Debug for ConcatSource {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let indent = f.width().unwrap_or(0);
    let indent_str = format!("{:indent$}", "", indent = indent);

    writeln!(f, "{indent_str}ConcatSource::new(vec![")?;

    let original_children = self.children.lock().unwrap();
    let children = match self.is_optimized.get() {
      Some(optimized_children) => optimized_children,
      None => original_children.as_ref(),
    };
    for child in children {
      writeln!(f, "{:indent$?},", child, indent = indent + 2)?;
    }
    write!(f, "{indent_str}]).boxed()")
  }
}

impl ConcatSource {
  /// Create a [ConcatSource] with [Source]s.
  pub fn new<S, T>(sources: S) -> Self
  where
    T: Source + 'static,
    S: IntoIterator<Item = T>,
  {
    let mut concat_source = ConcatSource::default();
    for source in sources {
      concat_source.add(source);
    }
    concat_source
  }

  fn optimized_children(&self) -> &[BoxSource] {
    self.is_optimized.get_or_init(|| {
      let mut children = self.children.lock().unwrap();
      optimize(&mut children)
    })
  }

  /// Get the (optimized) children of this concat source.
  pub fn children(&self) -> &[BoxSource] {
    self.optimized_children()
  }

  /// Add a [Source] to concat.
  pub fn add<S: Source + 'static>(&mut self, source: S) {
    // `&mut self` guarantees exclusive access, so the Mutex doesn't need to be
    // locked here — `get_mut` skips the atomic acquire/release. This matters
    // because `add` is called many hundreds of times per chunk in rspack's
    // code-generation hot path.
    let children = self
      .children
      .get_mut()
      .expect("ConcatSource children mutex poisoned");

    if let Some(optimized_children) = self.is_optimized.take() {
      *children = optimized_children;
    }

    // First check if it's already a BoxSource containing a ConcatSource
    if let Some(box_source) = source.as_any().downcast_ref::<BoxSource>() {
      if let Some(concat_source) = box_source.as_ref().as_any().downcast_ref::<ConcatSource>() {
        // Extend with existing children (cheap clone due to Arc)
        let original_children = concat_source.children.lock().unwrap();
        let other_children = match concat_source.is_optimized.get() {
          Some(optimized_children) => optimized_children,
          None => original_children.as_ref(),
        };
        children.extend(other_children.iter().cloned());
        return;
      }
    }

    // Check if the source itself is a ConcatSource
    if let Some(concat_source) = source.as_any().downcast_ref::<ConcatSource>() {
      // Extend with existing children (cheap clone due to Arc)
      let original_children = concat_source.children.lock().unwrap();
      let other_children = match concat_source.is_optimized.get() {
        Some(optimized_children) => optimized_children,
        None => original_children.as_ref(),
      };
      children.extend(other_children.iter().cloned());
    } else {
      // Regular source - box it and add to children
      children.push(source.boxed());
    }
  }
}

impl Source for ConcatSource {
  fn source(&self) -> SourceValue<'_> {
    let children = self.optimized_children();
    if children.len() == 1 {
      return children[0].source();
    }

    let mut string = String::with_capacity(self.size());
    let mut on_chunk = |chunk| {
      string.push_str(chunk);
    };
    children.iter().for_each(|child| {
      child.rope(&mut on_chunk);
    });
    SourceValue::String(Cow::Owned(string))
  }

  fn rope<'a>(&'a self, on_chunk: &mut dyn FnMut(&'a str)) {
    let children = self.optimized_children();
    children.iter().for_each(|child| {
      child.rope(on_chunk);
    });
  }

  fn buffer(&self) -> Cow<'_, [u8]> {
    let children = self.optimized_children();
    if children.len() == 1 {
      children[0].buffer()
    } else {
      // Use to_writer to avoid multiple heap allocations that would occur
      // when concatenating nested ConcatSource instances directly
      let mut buffer = Vec::with_capacity(self.size());
      self.to_writer(&mut buffer).unwrap();
      Cow::Owned(buffer)
    }
  }

  fn size(&self) -> usize {
    self
      .optimized_children()
      .iter()
      .map(|child| child.size())
      .sum()
  }

  fn map<'a>(&'a self, object_pool: &'a ObjectPool, options: &MapOptions) -> Option<SourceMap> {
    let chunks = self.stream_chunks();
    let result = get_map(object_pool, chunks.as_ref(), options);
    result
  }

  fn to_writer(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
    for child in self.optimized_children() {
      child.to_writer(writer)?;
    }
    Ok(())
  }
}

impl Hash for ConcatSource {
  fn hash<H: Hasher>(&self, state: &mut H) {
    "ConcatSource".hash(state);
    for child in self.optimized_children().iter() {
      child.hash(state);
    }
  }
}

impl PartialEq for ConcatSource {
  fn eq(&self, other: &Self) -> bool {
    self.optimized_children() == other.optimized_children()
  }
}
impl Eq for ConcatSource {}

struct ConcatSourceChunks<'source> {
  children_chunks: Vec<Box<dyn Chunks + 'source>>,
}

impl<'source> ConcatSourceChunks<'source> {
  fn new(concat_source: &'source ConcatSource) -> Self {
    let children = concat_source.optimized_children();
    let children_chunks = children
      .iter()
      .map(|child| child.stream_chunks())
      .collect::<Vec<_>>();
    Self { children_chunks }
  }
}

impl Chunks for ConcatSourceChunks<'_> {
  fn stream<'b>(
    &'b self,
    object_pool: &'b ObjectPool,
    options: &MapOptions,
    on_chunk: crate::helpers::OnChunk<'_, 'b>,
    on_source: crate::helpers::OnSource<'_, 'b>,
    on_name: crate::helpers::OnName<'_, 'b>,
  ) -> GeneratedInfo {
    if self.children_chunks.len() == 1 {
      return self.children_chunks[0].stream(object_pool, options, on_chunk, on_source, on_name);
    }
    let mut current_line_offset = 0;
    let mut current_column_offset = 0;
    let mut source_mapping: HashMap<Cow<str>, u32> = HashMap::default();
    let mut name_mapping: HashMap<Cow<str>, u32> = HashMap::default();
    let mut need_to_close_mapping = false;

    let source_index_mapping: RefCell<LinearMap<u32>> = RefCell::new(LinearMap::default());
    let name_index_mapping: RefCell<LinearMap<u32>> = RefCell::new(LinearMap::default());

    for child_handle in &self.children_chunks {
      source_index_mapping.borrow_mut().clear();
      name_index_mapping.borrow_mut().clear();
      let mut last_mapping_line = 0;
      let GeneratedInfo {
        generated_line,
        generated_column,
      } = child_handle.stream(
        object_pool,
        options,
        &mut |chunk, mapping| {
          let line = mapping.generated_line + current_line_offset;
          let column = if mapping.generated_line == 1 {
            mapping.generated_column + current_column_offset
          } else {
            mapping.generated_column
          };
          if need_to_close_mapping {
            if mapping.generated_line != 1 || mapping.generated_column != 0 {
              on_chunk(
                None,
                Mapping {
                  generated_line: current_line_offset + 1,
                  generated_column: current_column_offset,
                  original: None,
                },
              );
            }
            need_to_close_mapping = false;
          }
          let result_source_index = mapping.original.as_ref().and_then(|original| {
            source_index_mapping
              .borrow()
              .get(&original.source_index)
              .copied()
          });
          let result_name_index = mapping
            .original
            .as_ref()
            .and_then(|original| original.name_index)
            .and_then(|name_index| name_index_mapping.borrow().get(&name_index).copied());
          last_mapping_line = if result_source_index.is_none() {
            0
          } else {
            mapping.generated_line
          };
          if options.final_source {
            if let (Some(result_source_index), Some(original)) =
              (result_source_index, &mapping.original)
            {
              on_chunk(
                None,
                Mapping {
                  generated_line: line,
                  generated_column: column,
                  original: Some(OriginalLocation {
                    source_index: result_source_index,
                    original_line: original.original_line,
                    original_column: original.original_column,
                    name_index: result_name_index,
                  }),
                },
              );
            }
          } else if let (Some(result_source_index), Some(original)) =
            (result_source_index, &mapping.original)
          {
            on_chunk(
              chunk,
              Mapping {
                generated_line: line,
                generated_column: column,
                original: Some(OriginalLocation {
                  source_index: result_source_index,
                  original_line: original.original_line,
                  original_column: original.original_column,
                  name_index: result_name_index,
                }),
              },
            );
          } else {
            on_chunk(
              chunk,
              Mapping {
                generated_line: line,
                generated_column: column,
                original: None,
              },
            );
          }
        },
        &mut |i, source, source_content| {
          let mut global_index = source_mapping.get(&source).copied();
          if global_index.is_none() {
            let len = source_mapping.len() as u32;
            source_mapping.insert(source.clone(), len);
            on_source(len, source, source_content);
            global_index = Some(len);
          }
          source_index_mapping
            .borrow_mut()
            .insert(i, global_index.unwrap());
        },
        &mut |i, name| {
          let mut global_index = name_mapping.get(&name).copied();
          if global_index.is_none() {
            let len = name_mapping.len() as u32;
            name_mapping.insert(name.clone(), len);
            on_name(len, name);
            global_index = Some(len);
          }
          name_index_mapping
            .borrow_mut()
            .insert(i, global_index.unwrap());
        },
      );
      if need_to_close_mapping && (generated_line != 1 || generated_column != 0) {
        on_chunk(
          None,
          Mapping {
            generated_line: current_line_offset + 1,
            generated_column: current_column_offset,
            original: None,
          },
        );
        need_to_close_mapping = false;
      }
      if generated_line > 1 {
        current_column_offset = generated_column;
      } else {
        current_column_offset += generated_column;
      }
      need_to_close_mapping =
        need_to_close_mapping || (options.final_source && last_mapping_line == generated_line);
      current_line_offset += generated_line - 1;
    }
    GeneratedInfo {
      generated_line: current_line_offset + 1,
      generated_column: current_column_offset,
    }
  }
}

impl StreamChunks for ConcatSource {
  fn stream_chunks<'a>(&'a self) -> Box<dyn Chunks + 'a> {
    Box::new(ConcatSourceChunks::new(self))
  }
}

fn optimize(children: &mut Vec<BoxSource>) -> Vec<BoxSource> {
  let original_children = std::mem::take(children);

  if original_children.len() <= 1 {
    return original_children; // Nothing to optimize
  }

  let mut new_children = Vec::new();
  let mut current_raw_sources = Vec::new();

  for child in original_children {
    if child.as_ref().as_any().is::<RawStringSource>() {
      current_raw_sources.push(child);
    } else {
      // Flush any pending raw sources before adding the non-raw source
      merge_raw_sources(&mut current_raw_sources, &mut new_children);
      new_children.push(child);
    }
  }

  // Flush any remaining pending raw sources
  merge_raw_sources(&mut current_raw_sources, &mut new_children);

  new_children
}

/// Helper function to merge and flush pending raw sources.
fn merge_raw_sources(raw_sources: &mut Vec<BoxSource>, new_children: &mut Vec<BoxSource>) {
  match raw_sources.len() {
    0 => {} // Nothing to flush
    1 => {
      // Single source - move it directly
      new_children.push(raw_sources.pop().unwrap());
    }
    _ => {
      // Multiple sources - merge them
      let capacity = raw_sources.iter().map(|s| s.size()).sum();
      let mut merged_content = String::with_capacity(capacity);
      for source in raw_sources.drain(..) {
        merged_content.push_str(source.source().into_string_lossy().as_ref());
      }
      let merged_source = RawStringSource::from(merged_content);
      new_children.push(merged_source.boxed());
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{OriginalSource, RawBufferSource, RawStringSource};

  #[test]
  fn should_concat_two_sources() {
    let mut source = ConcatSource::new([
      RawStringSource::from("Hello World\n".to_string()).boxed(),
      OriginalSource::new(
        "console.log('test');\nconsole.log('test2');\n",
        "console.js",
      )
      .boxed(),
    ]);
    source.add(OriginalSource::new("Hello2\n", "hello.md"));

    let expected_source = "Hello World\nconsole.log('test');\nconsole.log('test2');\nHello2\n";
    assert_eq!(source.size(), 62);
    assert_eq!(source.source().into_string_lossy(), expected_source);
    assert_eq!(
      source
        .map(&ObjectPool::default(), &MapOptions::new(false))
        .unwrap(),
      SourceMap::from_json(
        r#"{
          "version": 3,
          "mappings": ";AAAA;AACA;ACDA",
          "names": [],
          "sources": ["console.js", "hello.md"],
          "sourcesContent": [
            "console.log('test');\nconsole.log('test2');\n",
            "Hello2\n"
          ]
        }"#,
      )
      .unwrap()
    );
    assert_eq!(
      source
        .map(&ObjectPool::default(), &MapOptions::default())
        .unwrap(),
      SourceMap::from_json(
        r#"{
          "version": 3,
          "mappings": ";AAAA;AACA;ACDA",
          "names": [],
          "sources": ["console.js", "hello.md"],
          "sourcesContent": [
            "console.log('test');\nconsole.log('test2');\n",
            "Hello2\n"
          ]
        }"#
      )
      .unwrap()
    );
  }

  #[test]
  fn should_concat_two_sources2() {
    let mut source = ConcatSource::new([
      RawStringSource::from("Hello World\n".to_string()).boxed(),
      OriginalSource::new(
        "console.log('test');\nconsole.log('test2');\n",
        "console.js",
      )
      .boxed(),
    ]);
    source.add(OriginalSource::new("Hello2\n", "hello.md"));

    let expected_source = "Hello World\nconsole.log('test');\nconsole.log('test2');\nHello2\n";
    assert_eq!(source.size(), 62);
    assert_eq!(source.source().into_string_lossy(), expected_source);
    assert_eq!(
      source
        .map(&ObjectPool::default(), &MapOptions::new(false))
        .unwrap(),
      SourceMap::from_json(
        r#"{
          "version": 3,
          "mappings": ";AAAA;AACA;ACDA",
          "names": [],
          "sources": ["console.js", "hello.md"],
          "sourcesContent": [
            "console.log('test');\nconsole.log('test2');\n",
            "Hello2\n"
          ]
        }"#,
      )
      .unwrap()
    );
    assert_eq!(
      source
        .map(&ObjectPool::default(), &MapOptions::default())
        .unwrap(),
      SourceMap::from_json(
        r#"{
          "version": 3,
          "mappings": ";AAAA;AACA;ACDA",
          "names": [],
          "sources": ["console.js", "hello.md"],
          "sourcesContent": [
            "console.log('test');\nconsole.log('test2');\n",
            "Hello2\n"
          ]
        }"#
      )
      .unwrap()
    );
  }

  #[test]
  fn should_concat_two_sources3() {
    let mut source = ConcatSource::new([
      RawBufferSource::from("Hello World\n".as_bytes()).boxed(),
      OriginalSource::new(
        "console.log('test');\nconsole.log('test2');\n",
        "console.js",
      )
      .boxed(),
    ]);
    source.add(OriginalSource::new("Hello2\n", "hello.md"));

    let expected_source = "Hello World\nconsole.log('test');\nconsole.log('test2');\nHello2\n";
    assert_eq!(source.size(), 62);
    assert_eq!(source.source().into_string_lossy(), expected_source);
    assert_eq!(
      source
        .map(&ObjectPool::default(), &MapOptions::new(false))
        .unwrap(),
      SourceMap::from_json(
        r#"{
          "version": 3,
          "mappings": ";AAAA;AACA;ACDA",
          "names": [],
          "sources": ["console.js", "hello.md"],
          "sourcesContent": [
            "console.log('test');\nconsole.log('test2');\n",
            "Hello2\n"
          ]
        }"#,
      )
      .unwrap()
    );
    assert_eq!(
      source
        .map(&ObjectPool::default(), &MapOptions::default())
        .unwrap(),
      SourceMap::from_json(
        r#"{
          "version": 3,
          "mappings": ";AAAA;AACA;ACDA",
          "names": [],
          "sources": ["console.js", "hello.md"],
          "sourcesContent": [
            "console.log('test');\nconsole.log('test2');\n",
            "Hello2\n"
          ]
        }"#
      )
      .unwrap()
    );
  }

  #[test]
  fn should_be_able_to_handle_strings_for_all_methods() {
    let mut source = ConcatSource::new([
      RawStringSource::from("Hello World\n".to_string()).boxed(),
      OriginalSource::new(
        "console.log('test');\nconsole.log('test2');\n",
        "console.js",
      )
      .boxed(),
    ]);
    let inner_source =
      ConcatSource::new([RawStringSource::from("("), "'string'".into(), ")".into()]);
    source.add(RawStringSource::from("console"));
    source.add(RawStringSource::from("."));
    source.add(RawStringSource::from("log"));
    source.add(inner_source);
    let expected_source =
      "Hello World\nconsole.log('test');\nconsole.log('test2');\nconsole.log('string')";
    let expected_map1 = SourceMap::from_json(
      r#"{
        "version": 3,
        "mappings": ";AAAA;AACA",
        "names": [],
        "sources": ["console.js"],
        "sourcesContent": ["console.log('test');\nconsole.log('test2');\n"]
      }"#,
    )
    .unwrap();
    assert_eq!(source.size(), 76);
    assert_eq!(source.source().into_string_lossy(), expected_source);
    assert_eq!(source.buffer(), expected_source.as_bytes());

    let map = source
      .map(&ObjectPool::default(), &MapOptions::new(false))
      .unwrap();
    assert_eq!(map, expected_map1);

    // TODO: test hash
  }

  #[test]
  fn should_return_null_as_map_when_only_generated_code_is_concatenated() {
    let source = ConcatSource::new([
      RawStringSource::from("Hello World\n"),
      RawStringSource::from("Hello World\n".to_string()),
      RawStringSource::from(""),
    ]);

    let result_text = source.source();
    let result_map = source.map(&ObjectPool::default(), &MapOptions::default());
    let result_list_map = source.map(&ObjectPool::default(), &MapOptions::new(false));

    assert_eq!(
      result_text.into_string_lossy(),
      "Hello World\nHello World\n"
    );
    assert!(result_map.is_none());
    assert!(result_list_map.is_none());
  }

  #[test]
  fn should_allow_to_concatenate_in_a_single_line() {
    let source = ConcatSource::new([
      OriginalSource::new("Hello", "hello.txt").boxed(),
      RawStringSource::from(" ").boxed(),
      OriginalSource::new("World ", "world.txt").boxed(),
      RawStringSource::from("is here\n").boxed(),
      OriginalSource::new("Hello\n", "hello.txt").boxed(),
      RawStringSource::from(" \n").boxed(),
      OriginalSource::new("World\n", "world.txt").boxed(),
      RawStringSource::from("is here").boxed(),
    ]);

    assert_eq!(
      source
        .map(&ObjectPool::default(), &MapOptions::default())
        .unwrap(),
      SourceMap::from_json(
        r#"{
          "mappings": "AAAA,K,CCAA,M;ADAA;;ACAA",
          "names": [],
          "sources": ["hello.txt", "world.txt"],
          "sourcesContent": ["Hello", "World "],
          "version": 3
        }"#
      )
      .unwrap(),
    );
    assert_eq!(
      source.source().into_string_lossy(),
      "Hello World is here\nHello\n \nWorld\nis here",
    );
  }

  #[test]
  fn should_allow_to_concat_buffer_sources() {
    let source = ConcatSource::new([
      RawStringSource::from("a"),
      RawStringSource::from("b"),
      RawStringSource::from("c"),
    ]);
    assert_eq!(source.source().into_string_lossy(), "abc");
    assert!(
      source
        .map(&ObjectPool::default(), &MapOptions::default())
        .is_none()
    );
  }

  #[test]
  fn should_flatten_nested_concat_sources() {
    let inner_concat = ConcatSource::new([
      RawStringSource::from("Hello "),
      RawStringSource::from("World"),
    ]);

    let outer_concat = ConcatSource::new([
      inner_concat.boxed(),
      RawStringSource::from("!").boxed(),
      ConcatSource::new([RawStringSource::from(" How"), RawStringSource::from(" are")]).boxed(),
      RawStringSource::from(" you?").boxed(),
    ]);

    assert_eq!(
      outer_concat.source().into_string_lossy(),
      "Hello World! How are you?"
    );
    // The key test: verify that nested ConcatSources are flattened
    // Should have 6 direct children instead of nested structure
    assert_eq!(outer_concat.optimized_children().len(), 1);
  }

  #[test]
  fn test_self_equality_no_deadlock() {
    let concat_source = ConcatSource::new([
      RawStringSource::from("Hello "),
      RawStringSource::from("World"),
    ])
    .boxed();
    assert_eq!(concat_source.as_ref(), concat_source.as_ref());

    concat_source.source();

    assert_eq!(concat_source.as_ref(), concat_source.as_ref());
  }

  #[test]
  fn test_debug_output() {
    let inner_concat = ConcatSource::new([
      RawStringSource::from("Hello "),
      RawStringSource::from("World"),
    ]);

    let mut outer_concat = ConcatSource::new([
      inner_concat.boxed(),
      RawStringSource::from("!").boxed(),
      ConcatSource::new([RawStringSource::from(" How"), RawStringSource::from(" are")]).boxed(),
      RawStringSource::from(" you?\n").boxed(),
    ]);

    assert_eq!(
      format!("{:?}", outer_concat),
      r#"ConcatSource::new(vec![
  RawStringSource::from_static("Hello ").boxed(),
  RawStringSource::from_static("World").boxed(),
  RawStringSource::from_static("!").boxed(),
  RawStringSource::from_static(" How").boxed(),
  RawStringSource::from_static(" are").boxed(),
  RawStringSource::from_static(" you?\n").boxed(),
]).boxed()"#
    );

    outer_concat.source();

    assert_eq!(
      format!("{:?}", outer_concat),
      r#"ConcatSource::new(vec![
  RawStringSource::from_static("Hello World! How are you?\n").boxed(),
]).boxed()"#
    );

    outer_concat.add(RawStringSource::from("I'm fine."));

    assert_eq!(
      format!("{:?}", outer_concat),
      r#"ConcatSource::new(vec![
  RawStringSource::from_static("Hello World! How are you?\n").boxed(),
  RawStringSource::from_static("I'm fine.").boxed(),
]).boxed()"#
    );

    outer_concat.source();

    assert_eq!(
      format!("{:?}", outer_concat),
      r#"ConcatSource::new(vec![
  RawStringSource::from_static("Hello World! How are you?\nI'm fine.").boxed(),
]).boxed()"#
    );
  }
}
