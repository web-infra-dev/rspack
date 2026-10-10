use std::{hash::Hasher, sync::Arc};

use rspack_cacheable::{enable_cacheable as cacheable, from_bytes, to_bytes, with::AsPreset};
use rspack_sources::{
  BoxSource, CachedSource, ConcatSource, MapOptions, ObjectPool, OriginalSource, RawBufferSource,
  RawStringSource, ReplaceSource, Source, SourceExt, SourceMap, SourceMapSource,
  WithoutOriginalOptions, stream_chunks::StreamChunks,
};
use rustc_hash::FxHasher;

#[cacheable]
#[derive(Debug)]
struct Data(#[cacheable(with=AsPreset)] BoxSource);

#[test]
fn test_rspack_source() {
  fn test_data(data: Data) {
    let bytes = to_bytes(&data, &()).unwrap();
    let new_data: Data = from_bytes(&bytes, &()).unwrap();
    assert_eq!(data.0.buffer(), new_data.0.buffer());
    assert_eq!(
      data.0.map(&ObjectPool::default(), &Default::default()),
      new_data.0.map(&ObjectPool::default(), &Default::default())
    );
  }

  test_data(Data(RawBufferSource::from("123".as_bytes()).boxed()));
  test_data(Data(RawStringSource::from_static("123").boxed()));
}

// Keep both cached module-level results and a cached final asset. ReplaceSource
// exercises non-final streams, while ConcatSource::map requests final streams.
fn nested_source() -> BoxSource {
  let code = "const value = '中文😀';\nconsole.log(value);\n";
  let original = OriginalSource::new(code, "original.js");
  let map = original
    .map(&ObjectPool::default(), &MapOptions::default())
    .expect("original map")
    .to_json();
  let module = CachedSource::new(SourceMapSource::new(WithoutOriginalOptions {
    value: code.to_string(),
    name: "original.js".to_string(),
    source_map: SourceMap::from_json(map).expect("parsed map"),
  }));
  let mut replaced = ReplaceSource::new(module);
  replaced.replace_static(6, 11, "renamed", Some("value"));
  replaced.insert_static(code.len() as u32, "// 尾😀\n", None);
  CachedSource::new(ConcatSource::new([
    RawStringSource::from_static("// head😀\n").boxed(),
    CachedSource::new(replaced).boxed(),
    CachedSource::new(RawStringSource::from_static("")).boxed(),
    CachedSource::new(OriginalSource::new("tail();", "tail.js")).boxed(),
  ]))
  .boxed()
}

fn hash(source: &BoxSource) -> u64 {
  let mut hasher = FxHasher::default();
  source.update_hash(&mut hasher);
  hasher.finish()
}

#[test]
fn test_nested_cached_source_roundtrip_and_repeated_reads() {
  let data = Data(nested_source());
  let cold_bytes = to_bytes(&data, &()).expect("serialize cold source");
  let cold_restored: Data = from_bytes(&cold_bytes, &()).expect("restore cold source");
  let pool = ObjectPool::default();
  let expected = data.0.buffer().into_owned();
  let expected_hash = hash(&data.0);
  for columns in [true, false] {
    let options = MapOptions::new(columns);
    let expected_map = data.0.map(&pool, &options).expect("final map").to_json();
    // Serialize both cold and warm results: memoized intermediate data is not
    // part of the persistent cache format and must not be required after restore.
    let bytes = to_bytes(&data, &()).expect("serialize source");
    let restored: Data = from_bytes(&bytes, &()).expect("restore source");
    for source in [&data.0, &cold_restored.0, &restored.0] {
      for _ in 0..3 {
        assert_eq!(source.source().as_bytes(), expected);
        assert_eq!(source.buffer().as_ref(), expected);
        assert_eq!(source.size(), expected.len());
        let mut rope = String::new();
        source.rope(&mut |chunk| rope.push_str(chunk));
        assert_eq!(rope.as_bytes(), expected);
        let mut written = Vec::new();
        source.to_writer(&mut written).expect("write source");
        assert_eq!(written, expected);
        assert_eq!(
          source.map(&pool, &options).expect("map").to_json(),
          expected_map
        );
        assert_eq!(hash(source), expected_hash);
      }
    }
  }
}

#[test]
fn test_cached_source_streaming_does_not_retain_intermediate_maps() {
  for columns in [true, false] {
    let options = MapOptions::new(columns);
    let pool = ObjectPool::default();
    let original = OriginalSource::new("a();\nb();\n", "module.js").boxed();
    let module = CachedSource::new(original.clone()).boxed();
    let mut replaced = ReplaceSource::new(module.clone());
    replaced.insert_static(0, "// banner\n", None);
    let asset = CachedSource::new(ConcatSource::new([
      RawStringSource::from_static("// asset\n").boxed(),
      CachedSource::new(replaced).boxed(),
    ]));
    let owners = Arc::strong_count(&original);
    let map = asset.map(&pool, &options).expect("asset map");
    // An intermediate cached map would hold an additional Arc to its source.
    assert_eq!(Arc::strong_count(&original), owners);
    assert_eq!(
      map.mappings().as_ptr(),
      asset
        .map(&pool, &options)
        .expect("cached asset map")
        .mappings()
        .as_ptr()
    );
    let expected = map.to_json();
    // A direct request still memoizes the module map, and streaming may reuse it.
    module.map(&pool, &options).expect("module map");
    assert_eq!(Arc::strong_count(&original), owners + 1);
    assert_eq!(
      asset
        .inner()
        .map(&pool, &options)
        .expect("warm stream")
        .to_json(),
      expected
    );
    let owner = module
      .clone()
      .map_static(&pool, &options)
      .expect("static map");
    drop(asset);
    drop(module);
    drop(original);
    assert_eq!(owner.sources_content()[0], "a();\nb();\n");
  }
}

#[test]
fn test_cached_source_replays_map_before_reading_source() {
  for columns in [true, false] {
    for (code, expected) in [
      ("const value = 'text';\n", "const renamed = 'text';\n"),
      ("const value = '中文😀';\n", "const renamed = '中文😀';\n"),
    ] {
      let pool = ObjectPool::default();
      let options = MapOptions::new(columns);
      let mut replaced =
        ReplaceSource::new(CachedSource::new(OriginalSource::new(code, "module.js")));
      replaced.replace_static(6, 11, "renamed", Some("value"));
      let source = CachedSource::new(replaced);
      let map = source.map(&pool, &options).expect("cached map");
      for _ in 0..3 {
        // Do not warm the source cache before replay. Each new handle must still
        // produce the same text while sharing the cached result across clones.
        let cloned = source.clone();
        let chunks = cloned.stream_chunks();
        let mut text = String::new();
        chunks.stream(
          &pool,
          &options,
          &mut |chunk, _| {
            if let Some(chunk) = chunk {
              text.push_str(chunk.as_str());
            }
          },
          &mut |_, _, _| {},
          &mut |_, _| {},
        );
        assert_eq!(text, expected);
        assert_eq!(
          cloned
            .map(&pool, &options)
            .expect("reused map")
            .mappings()
            .as_ptr(),
          map.mappings().as_ptr()
        );
      }
      assert_eq!(source.source().as_bytes(), expected.as_bytes());
    }
  }
}

#[test]
fn test_nested_cached_source_concurrent_maps_and_text() {
  let source = nested_source();
  let expected = source.buffer().into_owned();
  let expected_hash = hash(&source);
  std::thread::scope(|scope| {
    for columns in [true, false, true, false] {
      let source = source.clone();
      let expected = &expected;
      scope.spawn(move || {
        let pool = ObjectPool::default();
        let options = MapOptions::new(columns);
        for _ in 0..8 {
          assert_eq!(source.source().as_bytes(), expected);
          assert_eq!(hash(&source), expected_hash);
          let map = source.map(&pool, &options).expect("map");
          let restored: Data = from_bytes(
            &to_bytes(&Data(source.clone()), &()).expect("serialize"),
            &(),
          )
          .expect("restore");
          assert_eq!(restored.0.map(&pool, &options).expect("restored map"), map);
        }
      });
    }
  });
}

#[test]
fn test_cached_binary_source_roundtrip() {
  let data = Data(CachedSource::new(RawBufferSource::from(vec![0, 255, 128, 10])).boxed());
  let bytes = to_bytes(&data, &()).expect("serialize binary source");
  let restored: Data = from_bytes(&bytes, &()).expect("restore binary source");
  for source in [&data.0, &restored.0] {
    for columns in [true, false] {
      assert!(
        source
          .map(&ObjectPool::default(), &MapOptions::new(columns))
          .is_none()
      );
      assert!(source.source().is_buffer());
      assert_eq!(source.source().as_bytes(), [0, 255, 128, 10]);
      assert_eq!(source.buffer().as_ref(), [0, 255, 128, 10]);
      assert_eq!(source.size(), 4);
      // Replaying a cached None map also borrows temporary decoded text from
      // the Chunks handle; it must not change the binary source API.
      let chunks = source.stream_chunks();
      let mut streamed = Vec::new();
      chunks.stream(
        &ObjectPool::default(),
        &MapOptions::new(columns),
        &mut |chunk, _| {
          if let Some(chunk) = chunk {
            streamed.push(chunk.as_str());
          }
        },
        &mut |_, _, _| panic!("binary data has no source mapping"),
        &mut |_, _| panic!("binary data has no names"),
      );
      assert_eq!(streamed.concat(), "\0��\n");
      assert_eq!(source.buffer().as_ref(), [0, 255, 128, 10]);
      let mut rope = String::new();
      source.rope(&mut |chunk| rope.push_str(chunk));
      assert_eq!(rope, "\0��\n");
    }
  }
}
