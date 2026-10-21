use std::{
  borrow::Cow,
  hash::{Hash, Hasher},
  sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
  },
};

use rspack_sources::{
  CachedSource, ConcatSource, MapOptions, MappingSource, OriginalSource, ReplaceSource, Source,
  SourceCacheClearer, SourceExt, SourceMap, SourceMapSource, SourceValue, WithoutOriginalOptions,
  stream_chunks::{Chunks, StreamChunks},
};

#[derive(Debug)]
struct TrackedSource {
  inner: OriginalSource,
  rope_calls: Arc<AtomicUsize>,
}
impl PartialEq for TrackedSource {
  fn eq(&self, other: &Self) -> bool {
    self.inner == other.inner
  }
}
impl Eq for TrackedSource {}
impl Hash for TrackedSource {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.inner.hash(state);
  }
}
impl StreamChunks for TrackedSource {
  fn stream_chunks<'a>(&'a self) -> Box<dyn Chunks<'a> + 'a> {
    self.inner.stream_chunks()
  }
}
impl Source for TrackedSource {
  fn source(&self) -> SourceValue<'_> {
    self.inner.source()
  }
  fn rope<'a>(&'a self, on_chunk: &mut dyn FnMut(&'a str)) {
    self.rope_calls.fetch_add(1, Ordering::Relaxed);
    self.inner.rope(on_chunk);
  }
  fn buffer(&self) -> Cow<'_, [u8]> {
    self.inner.buffer()
  }
  fn size(&self) -> usize {
    self.inner.size()
  }
  fn map<'a>(&'a self, options: &MapOptions) -> Option<SourceMap<'a>> {
    self.inner.map(options)
  }
  fn map_static(self: Arc<Self>, options: &MapOptions) -> Option<SourceMap<'static>> {
    self.inner.clone().boxed().map_static(options)
  }
  fn to_writer(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
    self.inner.to_writer(writer)
  }
}

fn fixture() -> (CachedSource, Arc<AtomicUsize>) {
  let calls = Arc::new(AtomicUsize::new(0));
  let source = CachedSource::new(TrackedSource {
    inner: OriginalSource::new("const value = '中文😀';\nconsole.log(value);\n", "entry.js"),
    rope_calls: calls.clone(),
  });
  (source, calls)
}

#[test]
fn clear_shared_cache_and_rebuild_without_invalidating_maps() {
  let (source, calls) = fixture();
  let clone = source.clone();
  let expected = source.source().into_string_lossy().into_owned();
  clone.source();
  assert_eq!(calls.load(Ordering::Relaxed), 1);
  let options = MapOptions::new(true);
  let map = source.map(&options).expect("mapped source");
  let expected_map = map.to_json();
  let mut fragments = vec![];
  source.rope(&mut |chunk| fragments.push(chunk));

  clone.clear_cached_data();
  assert_eq!(map.to_json(), expected_map);
  assert_eq!(fragments.concat(), expected);
  assert_eq!(source.source().into_string_lossy(), expected);
  assert_eq!(calls.load(Ordering::Relaxed), 2);
  assert_eq!(
    source.map(&options).expect("rebuilt map").to_json(),
    expected_map
  );
  assert_eq!(source.size(), expected.len());
}

#[test]
fn forwarded_map_outlives_cache_and_source() {
  let (source, _) = fixture();
  let clone = source.clone();
  let wrapper = ReplaceSource::new(source).boxed();
  let map = wrapper
    .map_static(&MapOptions::new(true))
    .expect("forwarded map");
  let expected = map.to_json();
  clone.clear_cached_data();
  drop(clone);
  assert_eq!(map.to_json(), expected);
}

#[test]
fn stream_borrows_and_replay_survive_clearing() {
  let value = "const value = '中文😀';\n";
  let source = CachedSource::new(ConcatSource::new([SourceMapSource::new(
    WithoutOriginalOptions {
      value,
      name: "entry.js",
      source_map: SourceMap::new(
        "AAAAA",
        vec!["entry.js".into()],
        vec![value.into()],
        vec!["value".into()],
      ),
    },
  )
  .boxed()]));
  let options = MapOptions::new(true);
  let expected = source.map(&options).expect("mapped source").to_json();
  let mut contents = vec![];
  let mut names = vec![];
  let mut sources = vec![];
  {
    let chunks = source.stream_chunks();
    chunks.stream(
      &options,
      &mut |_, _| {},
      &mut |_, name, content| {
        sources.push(name);
        contents.extend(content);
      },
      &mut |_, name| names.push(name),
    );
  }
  source.clear_cached_data();
  assert_eq!(sources[0], "entry.js");
  assert!(contents[0].contains("中文😀"));
  assert_eq!(names[0], "value");
  assert_eq!(
    source.map(&options).expect("map after clear").to_json(),
    expected
  );
  let mut replay = vec![];
  source.stream_chunks().stream(
    &options,
    &mut |_, _| {},
    &mut |_, name, content| {
      replay.push((name.into_owned(), content.map(str::to_owned)));
    },
    &mut |_, _| {},
  );
  assert_eq!(replay[0].0, sources[0]);
  assert_eq!(replay[0].1.as_deref(), Some(contents[0]));
}

#[test]
fn traverse_shared_composites_and_source_map_owners() {
  let (source, calls) = fixture();
  let clone = source.clone();
  let original = source.boxed();
  let concat = ConcatSource::new([original.clone(), original.clone()]).boxed();
  let root = ReplaceSource::new(concat).boxed();
  original.source();
  let before = calls.load(Ordering::Relaxed);
  SourceCacheClearer::default().clear(&root);
  original.source();
  assert_eq!(calls.load(Ordering::Relaxed), before + 1);

  let map_root = ConcatSource::new([original.clone()]).boxed();
  let mapped =
    MappingSource::new(map_root.map_static(&MapOptions::new(true)).expect("map")).boxed();
  let json = mapped.source().into_string_lossy().into_owned();
  original.source();
  let before = calls.load(Ordering::Relaxed);
  SourceCacheClearer::default().clear(&mapped);
  original.source();
  assert_eq!(calls.load(Ordering::Relaxed), before + 1);
  assert_eq!(mapped.source().into_string_lossy(), json);
  assert_eq!(
    clone.source().into_string_lossy(),
    original.source().into_string_lossy()
  );
}

#[test]
fn concurrent_readers_keep_their_generations_alive() {
  let (source, _) = fixture();
  let source = Arc::new(source);
  let expected = source.source().into_string_lossy().into_owned();
  let expected_map = source.map(&MapOptions::new(true)).expect("map").to_json();
  std::thread::scope(|scope| {
    let source = &source;
    scope.spawn(move || {
      for _ in 0..500 {
        source.clear_cached_data();
      }
    });
    for _ in 0..4 {
      let expected = &expected;
      let expected_map = &expected_map;
      scope.spawn(move || {
        for _ in 0..100 {
          let map = source.map(&MapOptions::new(true)).expect("concurrent map");
          source.clear_cached_data();
          assert_eq!(map.to_json(), *expected_map);
          assert_eq!(source.source().into_string_lossy(), *expected);
        }
      });
    }
  });
}
