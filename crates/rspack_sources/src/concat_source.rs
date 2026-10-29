use std::{
  borrow::Cow,
  cell::RefCell,
  hash::{Hash, Hasher},
  sync::{Arc, Mutex, OnceLock},
};

use rustc_hash::FxHashMap as HashMap;

use crate::{
  BoxSource, CachedSource, MapOptions, MappingSource, OriginalSource, RawBufferSource,
  RawStringSource, ReplaceSource, Source, SourceExt, SourceMap, SourceMapSource, SourceValue,
  helpers::{Chunks, GeneratedInfo, StreamChunks, get_map, stream_chunks_of_raw_rope},
  linear_map::LinearMap,
  source::{Mapping, OriginalLocation},
};

/// Concatenate multiple [Source]s to a single [Source].
///
/// - [webpack-sources docs](https://github.com/webpack/webpack-sources/#concatsource).
///
/// ```
/// use rspack_sources::{
///   BoxSource, ConcatSource, MapOptions, OriginalSource, RawStringSource, Source, SourceExt,
///   SourceMap,
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
///   source.map(&MapOptions::new(false)).unwrap(),
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
///     }"#
///       .to_string(),
///   )
///   .unwrap()
/// );
/// ```
#[derive(Default)]
pub struct ConcatSource {
  children: Mutex<Vec<BoxSource>>,
  is_optimized: OnceLock<Vec<BoxSource>>,
  source_only: bool,
}

impl Clone for ConcatSource {
  fn clone(&self) -> Self {
    let children = self.children.lock().unwrap();
    Self {
      children: Mutex::new(children.clone()),
      is_optimized: match self.is_optimized.get() {
        Some(children) => {
          let once_lock = OnceLock::new();
          once_lock.get_or_init(|| children.clone());
          once_lock
        }
        None => OnceLock::default(),
      },
      source_only: self.source_only,
    }
  }
}

impl std::fmt::Debug for ConcatSource {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let indent = f.width().unwrap_or(0);
    let indent_str = format!("{:indent$}", "", indent = indent);

    let constructor = if self.source_only {
      "new_source_only"
    } else {
      "new"
    };
    writeln!(f, "{indent_str}ConcatSource::{constructor}(vec![")?;

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

  /// Preserve a generated text graph without exposing its source mappings.
  ///
  /// Devtool uses this after emitting the map separately. Known text sources keep
  /// their existing backing allocations. Binary and unknown sources retain the
  /// former `source()` materialization boundary, since their writer, buffer and
  /// lossy text views are not necessarily interchangeable.
  #[doc(hidden)]
  pub fn new_source_only(sources: impl IntoIterator<Item = BoxSource>) -> BoxSource {
    let sources = sources.into_iter().collect::<Vec<_>>();
    if !sources.iter().all(|source| has_text_bytes(source.as_ref())) {
      let mut children = sources
        .into_iter()
        .map(|source| match source.source() {
          SourceValue::String(text) => RawStringSource::from(text.into_owned()).boxed(),
          SourceValue::Buffer(bytes) => RawBufferSource::from(bytes.into_owned()).boxed(),
        })
        .collect::<Vec<_>>();
      return if children.len() == 1 {
        children.pop().expect("single normalized source")
      } else {
        Self::new(children).boxed()
      };
    }

    let mut children = Vec::with_capacity(sources.len());
    for source in sources {
      if let Some(concat) = source.as_any().downcast_ref::<Self>()
        && concat.source_only
      {
        // Both boundaries hide mappings, so the inner boundary is redundant.
        concat.with_children(|inner| children.extend(inner.iter().cloned()));
      } else {
        children.push(source);
      }
    }
    Self::from_parts(children, true).boxed()
  }

  /// Whether this concat is a map-free view of generated text.
  #[doc(hidden)]
  pub fn is_source_only(&self) -> bool {
    self.source_only
  }

  pub(crate) fn from_parts(children: Vec<BoxSource>, source_only: bool) -> Self {
    Self {
      children: Mutex::new(children),
      is_optimized: OnceLock::new(),
      source_only,
    }
  }

  fn with_children<T>(&self, visit: impl FnOnce(&[BoxSource]) -> T) -> T {
    if let Some(children) = self.is_optimized.get() {
      visit(children)
    } else {
      let children = self
        .children
        .lock()
        .expect("ConcatSource children mutex poisoned");
      // Optimization may have completed while this reader waited for the lock.
      match self.is_optimized.get() {
        Some(optimized_children) => visit(optimized_children),
        None => visit(&children),
      }
    }
  }

  fn optimized_children(&self) -> &[BoxSource] {
    if let Some(children) = self.is_optimized.get() {
      return children;
    }
    let mut children = self.children.lock().unwrap();
    // Keep the lock until publication so readers never see drained children
    // before the optimized list becomes available.
    self.is_optimized.get_or_init(|| optimize(&mut children))
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
      if let Some(concat_source) = box_source.as_ref().as_any().downcast_ref::<ConcatSource>()
        && !concat_source.source_only
      {
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
    if let Some(concat_source) = source.as_any().downcast_ref::<ConcatSource>()
      && !concat_source.source_only
    {
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

  fn map<'a>(&'a self, options: &MapOptions) -> Option<SourceMap<'a>> {
    if self.source_only {
      return None;
    }
    let chunks = self.stream_chunks();
    get_map(chunks.as_ref(), options).map(SourceMap::from_fields)
  }

  fn map_static(self: Arc<Self>, options: &MapOptions) -> Option<SourceMap<'static>> {
    if self.source_only {
      return None;
    }
    let owner = self.clone();
    self.as_ref().map(options).map(|map| map.into_static(owner))
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
    if self.source_only {
      "SourceOnlyConcatSource".hash(state);
    } else {
      "ConcatSource".hash(state);
    }
    for child in self.optimized_children().iter() {
      child.hash(state);
    }
  }
}

impl PartialEq for ConcatSource {
  fn eq(&self, other: &Self) -> bool {
    self.source_only == other.source_only && self.optimized_children() == other.optimized_children()
  }
}
impl Eq for ConcatSource {}

struct ConcatSourceChunks<'source> {
  children_chunks: Vec<Box<dyn Chunks<'source> + 'source>>,
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

impl<'source> Chunks<'source> for ConcatSourceChunks<'source> {
  fn stream<'chunk>(
    &'chunk self,
    options: &MapOptions,
    on_chunk: crate::helpers::OnChunk<'_, 'chunk>,
    on_source: crate::helpers::OnSource<'_, 'source>,
    on_name: crate::helpers::OnName<'_, 'source>,
  ) -> GeneratedInfo {
    if self.children_chunks.len() == 1 {
      return self.children_chunks[0].stream(options, on_chunk, on_source, on_name);
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
  fn stream_chunks<'a>(&'a self) -> Box<dyn Chunks<'a> + 'a> {
    if self.source_only {
      Box::new(SourceOnlyChunks(self))
    } else {
      Box::new(ConcatSourceChunks::new(self))
    }
  }
}

struct SourceOnlyChunks<'a>(&'a ConcatSource);

impl<'source> Chunks<'source> for SourceOnlyChunks<'source> {
  fn stream<'chunk>(
    &'chunk self,
    options: &MapOptions,
    on_chunk: crate::helpers::OnChunk<'_, 'chunk>,
    _on_source: crate::helpers::OnSource<'_, 'source>,
    _on_name: crate::helpers::OnName<'_, 'source>,
  ) -> GeneratedInfo {
    stream_chunks_of_raw_rope(self.0, options, on_chunk)
  }
}

fn has_text_bytes(source: &dyn Source) -> bool {
  let source = source.as_any();
  if source.is::<RawStringSource>()
    || source.is::<OriginalSource>()
    || source.is::<SourceMapSource>()
    || source.is::<MappingSource>()
  {
    return true;
  }
  if let Some(source) = source.downcast_ref::<ConcatSource>() {
    return source.with_children(|children| children.iter().all(|s| has_text_bytes(s.as_ref())));
  }
  if let Some(source) = source.downcast_ref::<CachedSource>() {
    return has_text_bytes(source.inner().as_ref());
  }
  if let Some(source) = source.downcast_ref::<ReplaceSource>() {
    return has_text_bytes(source.inner().as_ref());
  }
  false
}

fn optimize(children: &mut Vec<BoxSource>) -> Vec<BoxSource> {
  let original_children = std::mem::take(children);

  if original_children.len() <= 1 {
    return original_children; // Nothing to optimize
  }

  let mut new_children = Vec::with_capacity(original_children.len());
  let mut current_raw_sources = Vec::with_capacity(original_children.len());

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
      source.map(&MapOptions::new(false)).unwrap(),
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
          .to_string(),
      )
      .unwrap()
    );
    assert_eq!(
      source.map(&MapOptions::default()).unwrap(),
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
          .to_string()
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
      source.map(&MapOptions::new(false)).unwrap(),
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
          .to_string(),
      )
      .unwrap()
    );
    assert_eq!(
      source.map(&MapOptions::default()).unwrap(),
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
          .to_string()
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
      source.map(&MapOptions::new(false)).unwrap(),
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
          .to_string(),
      )
      .unwrap()
    );
    assert_eq!(
      source.map(&MapOptions::default()).unwrap(),
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
          .to_string()
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
      }"#
        .to_string(),
    )
    .unwrap();
    assert_eq!(source.size(), 76);
    assert_eq!(source.source().into_string_lossy(), expected_source);
    assert_eq!(source.buffer(), expected_source.as_bytes());

    let map = source.map(&MapOptions::new(false)).unwrap();
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
    let result_map = source.map(&MapOptions::default());
    let result_list_map = source.map(&MapOptions::new(false));

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
      source.map(&MapOptions::default()).unwrap(),
      SourceMap::from_json(
        r#"{
          "mappings": "AAAA,K,CCAA,M;ADAA;;ACAA",
          "names": [],
          "sources": ["hello.txt", "world.txt"],
          "sourcesContent": ["Hello", "World "],
          "version": 3
        }"#
          .to_string()
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
    assert!(source.map(&MapOptions::default()).is_none());
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
