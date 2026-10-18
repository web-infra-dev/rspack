use std::{
  borrow::Cow,
  cell::OnceCell,
  hash::{Hash, Hasher},
  sync::{
    Arc, OnceLock,
    atomic::{AtomicBool, Ordering},
  },
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
  chunks: ArcSwapOption<Box<[&'static str]>>,
  columns_map: ArcSwapOption<CachedMap>,
  line_only_map: ArcSwapOption<CachedMap>,
  source_maps_released: AtomicBool,
}

// The enum is itself stored behind an `Arc`; boxing the map would add another
// allocation without reducing the retained per-source footprint.
#[allow(clippy::large_enum_variant)]
pub(crate) enum CachedMap {
  None,
  Some(SourceMap<'static>),
}

impl CachedMap {
  fn into_source_map<'a>(self: Arc<Self>) -> Option<SourceMap<'a>> {
    let owner: Arc<dyn std::any::Any + Send + Sync> = self.clone();
    match self.as_ref() {
      CachedMap::None => None,
      CachedMap::Some(map) => {
        let fields = map.fields().as_borrowed();
        // SAFETY: `fields` borrows from `self`, and `owner` is a clone of the
        // same allocation retained by the returned source map.
        Some(unsafe { SourceMap::from_shared_fields(fields, owner) })
      }
    }
  }
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
  cache: Arc<CachedData>,
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
      cache: Arc::new(CachedData::default()),
    }
  }

  /// Get the inner source.
  pub fn inner(&self) -> &BoxSource {
    &self.inner
  }

  fn init_chunks(&self) {
    if self.cache.chunks.load().is_some() {
      return;
    }

    let mut chunks = Vec::new();
    self.inner.rope(&mut |chunk| chunks.push(chunk));
    let chunks = chunks.into_boxed_slice();
    #[allow(unsafe_code)]
    // SAFETY: CachedSource guarantees that the underlying source outlives the cache,
    // so transmuting Box<[&str]> to Box<[&'static str]> is safe in this context.
    // The boxed slice retains only the used references instead of Vec's spare capacity.
    let chunks = unsafe { std::mem::transmute::<Box<[&str]>, Box<[&'static str]>>(chunks) };
    self.cache.chunks.store(Some(Arc::new(chunks)));
  }

  fn with_chunks<R>(&self, on_chunks: impl FnOnce(&[&str]) -> R) -> R {
    self.init_chunks();
    let chunks = self.cache.chunks.load();
    on_chunks(
      chunks
        .as_ref()
        .expect("chunks initialized")
        .as_ref()
        .as_ref(),
    )
  }

  fn is_ascii(&self) -> bool {
    *self.cache.is_ascii.get_or_init(|| {
      let chunks = self.cache.chunks.load();
      if let Some(chunks) = chunks.as_ref() {
        return chunks.as_ref().iter().all(|chunk| chunk.is_ascii());
      }
      drop(chunks);
      let mut is_ascii = true;
      self.inner.rope(&mut |chunk| is_ascii &= chunk.is_ascii());
      is_ascii
    })
  }

  fn map_cache(&self, options: &MapOptions) -> &ArcSwapOption<CachedMap> {
    if options.columns {
      &self.cache.columns_map
    } else {
      &self.cache.line_only_map
    }
  }

  fn get_or_init_map(&self, options: &MapOptions) -> Arc<CachedMap> {
    let cache = self.map_cache(options);
    if let Some(map) = cache.load_full() {
      return map;
    }

    let map = Arc::new(match self.inner.clone().map_static(options) {
      Some(map) => CachedMap::Some(map),
      None => CachedMap::None,
    });
    self.store_map(options, &map);
    map
  }

  fn store_map(&self, options: &MapOptions, map: &Arc<CachedMap>) {
    if self.cache.source_maps_released.load(Ordering::Acquire) {
      return;
    }
    self.map_cache(options).store(Some(map.clone()));
    // A release may race with the store above. Recheck after publishing so a
    // map computed by a reader cannot become resident again.
    if self.cache.source_maps_released.load(Ordering::Acquire) {
      self.map_cache(options).store(None);
    }
  }
}

impl Source for CachedSource {
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

    self.with_chunks(|chunks| {
      let mut string =
        String::with_capacity(chunks.iter().fold(0, |size, chunk| size + chunk.len()));
      if self.cache.is_ascii.get().is_none() {
        let mut is_ascii = true;
        for chunk in chunks {
          if is_ascii {
            is_ascii = chunk.is_ascii();
          }
          string.push_str(chunk);
        }
        let _ = self.cache.is_ascii.set(is_ascii);
      } else {
        for chunk in chunks {
          string.push_str(chunk);
        }
      }
      SourceValue::String(Cow::Owned(string))
    })
  }

  fn rope<'a>(&'a self, on_chunk: &mut dyn FnMut(&'a str)) {
    self.init_chunks();
    let chunks = self.cache.chunks.load();
    for chunk in chunks.as_ref().expect("chunks initialized").as_ref().iter() {
      on_chunk(chunk);
    }
  }

  fn buffer(&self) -> Cow<'_, [u8]> {
    self.inner.buffer()
  }

  fn size(&self) -> usize {
    *self.cache.size.get_or_init(|| {
      let chunks = self.cache.chunks.load();
      if let Some(chunks) = chunks.as_ref() {
        return chunks
          .as_ref()
          .iter()
          .fold(0, |acc, chunk| acc + chunk.len());
      }
      self.inner.size()
    })
  }

  fn map<'a>(&'a self, options: &MapOptions) -> Option<SourceMap<'a>> {
    self.get_or_init_map(options).into_source_map()
  }

  fn map_static(self: Arc<Self>, options: &MapOptions) -> Option<SourceMap<'static>> {
    self.get_or_init_map(options).into_source_map()
  }

  fn to_writer(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
    self.inner.to_writer(writer)
  }

  fn release_caches(&self) {
    self.cache.chunks.store(None);
    self
      .cache
      .source_maps_released
      .store(true, Ordering::Release);
    self.cache.columns_map.store(None);
    self.cache.line_only_map.store(None);
    self.inner.release_caches();
  }
}

struct CachedSourceChunks<'source> {
  cache_source: &'source CachedSource,
  chunks: OnceCell<Box<dyn Chunks<'source> + 'source>>,
  source: OnceCell<Cow<'source, str>>,
  maps: [OnceCell<Arc<CachedMap>>; 2],
}

impl<'source> CachedSourceChunks<'source> {
  fn new(cache_source: &'source CachedSource) -> Self {
    Self {
      cache_source,
      chunks: OnceCell::new(),
      source: OnceCell::new(),
      maps: Default::default(),
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
    let map_index = usize::from(options.columns);
    let cached = if let Some(cached) = self.maps[map_index].get() {
      cached.clone()
    } else if let Some(cached) = self.cache_source.map_cache(options).load_full() {
      let _ = self.maps[map_index].set(cached.clone());
      cached
    } else {
      let (generated_info, map) = stream_and_get_source_and_map(
        options,
        self.get_or_init_chunks(),
        on_chunk,
        on_source,
        on_name,
      );
      let map = Arc::new(match map {
        Some(map) => {
          CachedMap::Some(SourceMap::from_fields(map).into_static(self.cache_source.inner.clone()))
        }
        None => CachedMap::None,
      });
      self.cache_source.store_map(options, &map);
      let _ = self.maps[map_index].set(map);
      return generated_info;
    };
    #[allow(unsafe_code)]
    // SAFETY: `self.maps` owns the `Arc` for the full `'source` lifetime, so
    // extending this borrow to `'source` does not outlive its allocation.
    let cached: &'source CachedMap = unsafe { &*(cached.as_ref() as *const CachedMap) };
    match cached {
      CachedMap::Some(map) => {
        #[allow(unsafe_code)]
        // SAFETY: the source cell is owned by `self`, which lives for `'source`.
        let source = unsafe {
          std::mem::transmute::<TextSpan<'_>, TextSpan<'source>>(self.get_or_init_source())
        };
        stream_chunks_of_source_map(options, source, map.fields(), on_chunk, on_source, on_name)
      }
      CachedMap::None => {
        stream_chunks_of_raw_rope(self.cache_source.inner.as_ref(), options, on_chunk)
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
    (self.cache.hash.get_or_init(|| {
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

    let cached_map = clone.cache.columns_map.load_full().unwrap();
    let CachedMap::Some(cached_map) = cached_map.as_ref() else {
      panic!("expected a cached source map");
    };
    let map = source.map(&map_options).unwrap();
    assert!(cached_map.fields() == map.fields());
  }

  #[test]
  fn released_maps_are_not_restored_by_lazy_rebuilds() {
    let source = CachedSource::new(OriginalSource::new("console.log(1);", "index.js"));
    let options = MapOptions::default();

    assert!(source.map(&options).is_some());
    source.release_caches();
    assert!(source.map(&options).is_some());

    assert!(source.cache.columns_map.load().is_none());
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
