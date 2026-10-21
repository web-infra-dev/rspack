use std::{
  borrow::Cow,
  cell::OnceCell,
  hash::{Hash, Hasher},
  sync::{Arc, OnceLock},
};

use arc_swap::ArcSwapOption;
use rustc_hash::FxHasher;

use crate::{
  BoxSource, MapOptions, RawBufferSource, Source, SourceExt, SourceMap,
  helpers::{
    Chunks, GeneratedInfo, StreamChunks, TextSpan, stream_and_get_source_and_map,
    stream_chunks_of_raw_rope, stream_chunks_of_source_map,
  },
  source::SourceValue,
};

#[derive(Default)]
struct CachedData {
  hash: OnceLock<u64>,
  size: OnceLock<usize>,
  is_ascii: OnceLock<bool>,
  chunks: OnceLock<Box<[&'static str]>>,
  columns_map: OnceLock<Option<Arc<SourceMap<'static>>>>,
  line_only_map: OnceLock<Option<Arc<SourceMap<'static>>>>,
}

// Streaming callbacks can borrow map fields for the lifetime of the source.
// Keep only those maps stable; they must not pin the evictable rope/hash cache.
#[derive(Default)]
struct CacheState {
  data: ArcSwapOption<CachedData>,
  streamed_columns_map: OnceLock<Option<Arc<SourceMap<'static>>>>,
  streamed_line_only_map: OnceLock<Option<Arc<SourceMap<'static>>>>,
}

/// It tries to reused cached results from other methods to avoid calculations,
/// usually used after modify is finished.
///
/// - [webpack-sources docs](https://github.com/webpack/webpack-sources/#cachedsource).
///
/// ```
/// use rspack_sources::{
///   BoxSource, CachedSource, ConcatSource, MapOptions, OriginalSource, RawStringSource, Source,
///   SourceExt, SourceMap,
/// };
///
/// let mut concat = ConcatSource::new([
///   RawStringSource::from("Hello World\n".to_string()).boxed(),
///   OriginalSource::new(
///     "console.log('test');\nconsole.log('test2');\n",
///     "console.js",
///   )
///   .boxed(),
/// ]);
/// concat.add(OriginalSource::new("Hello2\n", "hello.md"));
///
/// let cached = CachedSource::new(concat);
///
/// assert_eq!(
///   cached.source().into_string_lossy(),
///   "Hello World\nconsole.log('test');\nconsole.log('test2');\nHello2\n"
/// );
/// // second time will be fast.
/// assert_eq!(
///   cached.source().into_string_lossy(),
///   "Hello World\nconsole.log('test');\nconsole.log('test2');\nHello2\n"
/// );
/// ```
pub struct CachedSource {
  inner: BoxSource,
  cache: Arc<CacheState>,
}

impl CachedSource {
  /// Create a [CachedSource] with the original [Source].
  pub fn new<T: SourceExt>(inner: T) -> Self {
    let box_source = inner.boxed();
    // Check if it's already a BoxSource containing a CachedSource
    if let Some(cached_source) = box_source.as_ref().as_any().downcast_ref::<CachedSource>() {
      return cached_source.clone();
    }

    Self {
      inner: box_source,
      cache: Arc::new(CacheState::default()),
    }
  }

  /// Get the inner source.
  pub fn inner(&self) -> &BoxSource {
    &self.inner
  }

  fn cached_data(&self) -> Arc<CachedData> {
    if let Some(data) = self.cache.data.load_full() {
      return data;
    }
    let data = Arc::new(CachedData::default());
    let previous = self
      .cache
      .data
      .compare_and_swap(&None::<Arc<CachedData>>, Some(Arc::clone(&data)));
    previous.as_ref().cloned().unwrap_or(data)
  }

  /// Discard memoized results without changing the underlying source.
  /// Existing map views and in-flight readers retain their own owners.
  pub fn clear_cached_data(&self) {
    self.cache.data.store(None);
  }

  fn get_or_init_chunks<'a>(&self, cache: &'a CachedData) -> &'a [&'static str] {
    cache.chunks.get_or_init(|| {
      let mut chunks = Vec::new();
      self.inner.rope(&mut |chunk| {
        chunks.push(chunk);
      });
      let chunks = chunks.into_boxed_slice();
      #[allow(unsafe_code)]
      // SAFETY: CachedSource guarantees that the underlying source outlives the cache,
      // so transmuting Box<[&str]> to Box<[&'static str]> is safe in this context.
      // The boxed slice retains only the used references instead of Vec's spare capacity.
      unsafe {
        std::mem::transmute::<Box<[&str]>, Box<[&'static str]>>(chunks)
      }
    })
  }

  fn is_ascii(&self) -> bool {
    let cache = self.cached_data();
    *cache.is_ascii.get_or_init(|| {
      if let Some(chunks) = cache.chunks.get() {
        return chunks.iter().all(|chunk| chunk.is_ascii());
      }
      let mut is_ascii = true;
      self.inner.rope(&mut |chunk| is_ascii &= chunk.is_ascii());
      is_ascii
    })
  }

  fn get_or_init_map<'a>(&self, options: &MapOptions) -> Option<SourceMap<'a>> {
    let cache = self.cached_data();
    let (cell, streamed) = if options.columns {
      (&cache.columns_map, &self.cache.streamed_columns_map)
    } else {
      (&cache.line_only_map, &self.cache.streamed_line_only_map)
    };
    cell
      .get_or_init(|| {
        if let Some(map) = streamed.get() {
          return map.clone();
        }
        self.inner.clone().map_static(options).map(Arc::new)
      })
      .as_ref()
      .map(|map| SourceMap::from_shared(Arc::clone(map)))
  }
}

impl Source for CachedSource {
  fn clear_cached_sources(&self, clearer: &mut crate::SourceCacheClearer) {
    self.clear_cached_data();
    clearer.clear(&self.inner);
  }

  fn source(&self) -> SourceValue<'_> {
    // Check if it's a RawBufferSource containing a CachedSource
    if let Some(buffer_source) = self
      .inner
      .as_ref()
      .as_any()
      .downcast_ref::<RawBufferSource>()
    {
      return buffer_source.source();
    }

    let cache = self.cached_data();
    let chunks = self.get_or_init_chunks(&cache);
    let size = *cache
      .size
      .get_or_init(|| chunks.iter().map(|chunk| chunk.len()).sum());
    let mut string = String::with_capacity(size);
    if cache.is_ascii.get().is_none() {
      let mut is_ascii = true;
      for chunk in chunks {
        if is_ascii {
          is_ascii = chunk.is_ascii();
        }
        string.push_str(chunk);
      }
      let _ = cache.is_ascii.set(is_ascii);
    } else {
      for chunk in chunks {
        string.push_str(chunk);
      }
    }
    SourceValue::String(Cow::Owned(string))
  }

  fn rope<'a>(&'a self, on_chunk: &mut dyn FnMut(&'a str)) {
    let cache = self.cached_data();
    let chunks = self.get_or_init_chunks(&cache);
    chunks.iter().for_each(|chunk| on_chunk(chunk));
  }

  fn buffer(&self) -> Cow<'_, [u8]> {
    self.inner.buffer()
  }

  fn size(&self) -> usize {
    let cache = self.cached_data();
    *cache.size.get_or_init(|| {
      if let Some(chunks) = cache.chunks.get() {
        return chunks.iter().fold(0, |acc, chunk| acc + chunk.len());
      }
      self.inner.size()
    })
  }

  fn map<'a>(&'a self, options: &MapOptions) -> Option<SourceMap<'a>> {
    self.get_or_init_map(options)
  }

  fn map_static(self: Arc<Self>, options: &MapOptions) -> Option<SourceMap<'static>> {
    self.get_or_init_map(options)
  }

  fn to_writer(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
    self.inner.to_writer(writer)
  }
}

struct CachedSourceChunks<'source> {
  cache_source: &'source CachedSource,
  chunks: OnceCell<Box<dyn Chunks<'source> + 'source>>,
  source: OnceCell<Cow<'source, str>>,
}

impl<'source> CachedSourceChunks<'source> {
  fn new(cache_source: &'source CachedSource) -> Self {
    Self {
      cache_source,
      chunks: OnceCell::new(),
      source: OnceCell::new(),
    }
  }

  fn get_or_init_chunks(&self) -> &dyn Chunks<'source> {
    self
      .chunks
      .get_or_init(|| self.cache_source.inner.stream_chunks())
      .as_ref()
  }

  fn get_or_init_source(&self) -> TextSpan<'_> {
    let source = self
      .source
      .get_or_init(|| self.cache_source.source().into_string_lossy());
    let is_ascii = self.cache_source.is_ascii();
    TextSpan::with_known(source.as_ref(), is_ascii)
  }
}

impl<'source> Chunks<'source> for CachedSourceChunks<'source> {
  fn stream<'chunk>(
    &'chunk self,
    options: &MapOptions,
    on_chunk: crate::helpers::OnChunk<'_, 'chunk>,
    on_source: crate::helpers::OnSource<'_, 'source>,
    on_name: crate::helpers::OnName<'_, 'source>,
  ) -> GeneratedInfo {
    let cache = self.cache_source.cached_data();
    let (cell, cached_cell) = if options.columns {
      (
        &self.cache_source.cache.streamed_columns_map,
        &cache.columns_map,
      )
    } else {
      (
        &self.cache_source.cache.streamed_line_only_map,
        &cache.line_only_map,
      )
    };
    if let Some(map) = cached_cell.get() {
      cell.get_or_init(|| map.clone());
    }
    match cell.get() {
      Some(Some(map)) => {
        let source = self.get_or_init_source();
        stream_chunks_of_source_map(options, source, map.fields(), on_chunk, on_source, on_name)
      }
      Some(None) => stream_chunks_of_raw_rope(self.cache_source.inner.as_ref(), options, on_chunk),
      None => {
        let (generated_info, map) = stream_and_get_source_and_map(
          options,
          self.get_or_init_chunks(),
          on_chunk,
          on_source,
          on_name,
        );
        let map = cell.get_or_init(|| {
          map.map(|map| {
            Arc::new(SourceMap::from_fields(map).into_static(self.cache_source.inner.clone()))
          })
        });
        cached_cell.get_or_init(|| map.clone());
        generated_info
      }
    }
  }
}

impl StreamChunks for CachedSource {
  fn stream_chunks<'a>(&'a self) -> Box<dyn Chunks<'a> + 'a> {
    Box::new(CachedSourceChunks::new(self))
  }
}

impl Clone for CachedSource {
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
      cache: self.cache.clone(),
    }
  }
}

impl Hash for CachedSource {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    let cache = self.cached_data();
    (cache.hash.get_or_init(|| {
      let mut hasher = FxHasher::default();
      self.inner.hash(&mut hasher);
      hasher.finish()
    }))
    .hash(state);
  }
}

impl PartialEq for CachedSource {
  fn eq(&self, other: &Self) -> bool {
    self.inner.as_ref() == other.inner.as_ref()
  }
}

impl Eq for CachedSource {}

impl std::fmt::Debug for CachedSource {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
    let indent = f.width().unwrap_or(0);
    let indent_str = format!("{:indent$}", "", indent = indent);

    writeln!(f, "{indent_str}CachedSource::new(")?;
    writeln!(
      f,
      "{indent_str}{:indent$?}",
      self.inner,
      indent = indent + 2
    )?;
    write!(f, "{indent_str}).boxed()")
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    ConcatSource, OriginalSource, RawBufferSource, RawStringSource, ReplaceSource, SourceExt,
    SourceMapSource, WithoutOriginalOptions,
  };

  #[test]
  fn line_number_should_not_add_one() {
    let source = ConcatSource::new([
      CachedSource::new(RawStringSource::from("\n")).boxed(),
      SourceMapSource::new(WithoutOriginalOptions {
        value: "\nconsole.log(1);\n".to_string(),
        name: "index.js".to_string(),
        source_map: SourceMap::new(
          ";AACA",
          vec!["index.js".into()],
          vec!["// DELETE IT\nconsole.log(1)".into()],
          vec![],
        ),
      })
      .boxed(),
    ]);
    let map = source.map(&Default::default()).unwrap();
    assert_eq!(map.mappings(), ";;AACA");
  }

  #[test]
  fn should_allow_to_store_and_share_cached_data() {
    let original = OriginalSource::new("Hello World", "test.txt");
    let source = CachedSource::new(original);
    let clone = source.clone();

    // fill up cache
    let map_options = MapOptions::default();
    source.source();
    source.buffer();
    source.size();
    source.map(&map_options);

    let cache = clone.cached_data();
    let cached_map = cache.columns_map.get().and_then(Option::as_ref).unwrap();
    let map = source.map(&map_options).unwrap();
    assert!(cached_map.fields() == map.fields());
  }

  #[test]
  fn should_return_the_correct_size_for_binary_files() {
    let source = OriginalSource::new(String::from_utf8(vec![0; 256]).unwrap(), "file.wasm");
    let cached_source = CachedSource::new(source);

    assert_eq!(cached_source.size(), 256);
    assert_eq!(cached_source.size(), 256);
  }

  #[test]
  fn should_return_the_correct_size_for_cached_binary_files() {
    let source = OriginalSource::new(String::from_utf8(vec![0; 256]).unwrap(), "file.wasm");
    let cached_source = CachedSource::new(source);

    cached_source.source();
    assert_eq!(cached_source.size(), 256);
    assert_eq!(cached_source.size(), 256);
  }

  #[test]
  fn should_return_the_correct_size_for_text_files() {
    let source = OriginalSource::new("TestTestTest", "file.js");
    let cached_source = CachedSource::new(source);

    assert_eq!(cached_source.size(), 12);
    assert_eq!(cached_source.size(), 12);
  }

  #[test]
  fn should_return_the_correct_size_for_cached_text_files() {
    let source = OriginalSource::new("TestTestTest", "file.js");
    let cached_source = CachedSource::new(source);

    cached_source.source();
    assert_eq!(cached_source.size(), 12);
    assert_eq!(cached_source.size(), 12);
  }

  #[test]
  fn should_produce_correct_output_for_cached_raw_source() {
    let map_options = MapOptions::new(true);

    let source = RawStringSource::from("Test\nTest\nTest\n");
    let mut on_chunk_count = 0;
    let mut on_source_count = 0;
    let mut on_name_count = 0;
    let generated_info = {
      let chunks = source.stream_chunks();
      chunks.stream(
        &map_options,
        &mut |_chunk, _mapping| {
          on_chunk_count += 1;
        },
        &mut |_source_index, _source, _source_content| {
          on_source_count += 1;
        },
        &mut |_name_index, _name| {
          on_name_count += 1;
        },
      )
    };

    let cached_source = CachedSource::new(source);
    cached_source.stream_chunks().stream(
      &map_options,
      &mut |_chunk, _mapping| {},
      &mut |_source_index, _source, _source_content| {},
      &mut |_name_index, _name| {},
    );

    let mut cached_on_chunk_count = 0;
    let mut cached_on_source_count = 0;
    let mut cached_on_name_count = 0;
    let cached_generated_info = cached_source.stream_chunks().stream(
      &map_options,
      &mut |_chunk, _mapping| {
        cached_on_chunk_count += 1;
      },
      &mut |_source_index, _source, _source_content| {
        cached_on_source_count += 1;
      },
      &mut |_name_index, _name| {
        cached_on_name_count += 1;
      },
    );

    assert_eq!(on_chunk_count, cached_on_chunk_count);
    assert_eq!(on_source_count, cached_on_source_count);
    assert_eq!(on_name_count, cached_on_name_count);
    assert_eq!(generated_info, cached_generated_info);
  }

  #[test]
  fn should_have_correct_buffer_if_cache_buffer_from_cache_source() {
    let buf = vec![128u8];
    let source = CachedSource::new(RawBufferSource::from(buf.clone()));

    source.source();
    assert_eq!(source.buffer(), buf.as_slice());
  }

  #[test]
  fn hash_should_different_when_map_are_different() {
    let hash1 = {
      let mut source = ReplaceSource::new(OriginalSource::new("Hello", "hello.txt").boxed());
      source.insert_static(5, " world", None);
      let cache = CachedSource::new(source);
      let mut hasher = FxHasher::default();
      cache.hash(&mut hasher);
      hasher.finish()
    };

    let hash2 = {
      let source = OriginalSource::new("Hello world", "hello.txt").boxed();
      let cache = CachedSource::new(source);
      let mut hasher = FxHasher::default();
      cache.hash(&mut hasher);
      hasher.finish()
    };

    assert!(hash1 != hash2);
  }

  #[test]
  fn size_over_a_raw_buffer_source() {
    // buffer from PNG
    let raw = RawBufferSource::from(vec![137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13]);
    let raw_size = raw.size();
    let cached = CachedSource::new(raw.boxed());
    let cached_size = cached.size();
    assert_eq!(raw_size, cached_size);
  }
}
