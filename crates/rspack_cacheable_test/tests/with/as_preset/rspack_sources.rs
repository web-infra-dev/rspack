use std::{
  collections::hash_map::DefaultHasher,
  hash::{Hash, Hasher},
  sync::{Arc, Barrier},
  thread,
};

use rspack_cacheable::{enable_cacheable as cacheable, from_bytes, to_bytes, with::AsPreset};
use rspack_sources::{
  BoxSource, CachedSource, ConcatSource, OriginalSource, RawBufferSource, RawStringSource,
  ReplaceSource, Source, SourceExt,
};

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
      data.0.map(&Default::default()),
      new_data.0.map(&Default::default())
    );
  }

  test_data(Data(RawBufferSource::from("123".as_bytes()).boxed()));
  test_data(Data(RawStringSource::from_static("123").boxed()));
}

#[test]
fn source_only_concat_shares_text_and_keeps_mapping_boundary() {
  let inner = CachedSource::new(ConcatSource::new([
    OriginalSource::new("😀abc\n", "hidden.js").boxed(),
    RawStringSource::from_static("😀尾巴").boxed(),
  ]))
  .boxed();
  let hidden = ConcatSource::new_source_only([Arc::clone(&inner)]);
  let concat = hidden.as_any().downcast_ref::<ConcatSource>().unwrap();
  assert!(concat.is_source_only());
  assert!(Arc::ptr_eq(&concat.children()[0], &inner));
  assert!(hidden.map(&Default::default()).is_none());
  assert!(
    Arc::clone(&hidden)
      .map_static(&Default::default())
      .is_none()
  );

  let wrap = |body| {
    let mut source = ReplaceSource::new(
      ConcatSource::new([
        OriginalSource::new("visible();\n", "visible.js").boxed(),
        body,
        OriginalSource::new("end();", "tail.js").boxed(),
      ])
      .boxed(),
    );
    source.insert(0, "/* begin */".to_string(), None);
    source.boxed()
  };
  let actual = wrap(hidden);
  let expected = wrap(RawStringSource::from("😀abc\n😀尾巴").boxed());
  assert_eq!(actual.buffer(), expected.buffer());
  for columns in [false, true] {
    let options = rspack_sources::MapOptions::new(columns);
    let map = actual.map(&options).unwrap();
    assert_eq!(map.sources(), ["visible.js", "tail.js"]);
    let tail = map
      .decoded_mappings()
      .find(|mapping| {
        mapping
          .original
          .is_some_and(|original| original.source_index == 1)
      })
      .unwrap();
    assert_eq!(tail.generated_line, 3);
    // Full maps count 😀 as two UTF-16 code units; line-only maps omit columns.
    assert_eq!(tail.generated_column, if columns { 4 } else { 0 });
  }
}

#[test]
fn source_only_concat_cache_round_trip() {
  let source =
    ConcatSource::new_source_only([
      OriginalSource::new("module.exports = '😀';", "original.js").boxed()
    ]);
  let source = ConcatSource::new_source_only([
    source,
    RawStringSource::from_static("\n//# sourceMappingURL=out.js.map").boxed(),
  ]);
  let data = Data(source);
  let bytes = to_bytes(&data, &()).unwrap();
  let restored: Data = from_bytes(&bytes, &()).unwrap();
  assert!(
    restored
      .0
      .as_any()
      .downcast_ref::<ConcatSource>()
      .unwrap()
      .is_source_only()
  );
  assert!(data.0.eq(&restored.0));
  assert_eq!(data.0.buffer(), restored.0.buffer());
  assert!(restored.0.map(&Default::default()).is_none());
  let hash = |source: &BoxSource| {
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);
    hasher.finish()
  };
  assert_eq!(hash(&data.0), hash(&restored.0));
}

#[test]
fn source_only_concat_preserves_binary_materialization() {
  let binary = RawBufferSource::from(vec![0xff, 0, b'x']).boxed();
  let empty_replace = ReplaceSource::new(Arc::clone(&binary)).boxed();
  let mixed = ConcatSource::new([
    Arc::clone(&binary),
    OriginalSource::new("tail", "tail.js").boxed(),
  ])
  .boxed();
  for source in [binary, empty_replace, mixed] {
    let expected = source.source().as_bytes().to_vec();
    let normalized = ConcatSource::new_source_only([source]);
    assert_eq!(normalized.buffer().as_ref(), expected);
    assert!(normalized.map(&Default::default()).is_none());
  }
}

#[test]
fn source_only_concat_concurrent_wrapping_preserves_content() {
  const CHILDREN: usize = 4096;
  const ROUNDS: usize = 64;
  let fragment = RawStringSource::from_static("generated 😀\n").boxed();
  let expected = "generated 😀\n".repeat(CHILDREN);

  for _ in 0..ROUNDS {
    // Keep the concat unoptimized until readers and constructors race below.
    let source = ConcatSource::new_source_only((0..CHILDREN).map(|_| Arc::clone(&fragment)));
    let barrier = Barrier::new(5);
    thread::scope(|scope| {
      scope.spawn(|| {
        barrier.wait();
        assert!(source.buffer().as_ref() == expected.as_bytes());
      });
      scope.spawn(|| {
        barrier.wait();
        let cloned = source
          .as_any()
          .downcast_ref::<ConcatSource>()
          .unwrap()
          .clone();
        assert!(cloned.buffer().as_ref() == expected.as_bytes());
      });
      for _ in 0..3 {
        scope.spawn(|| {
          barrier.wait();
          let wrapped = ConcatSource::new_source_only([Arc::clone(&source)]);
          let actual = wrapped.buffer();
          assert_eq!(actual.len(), expected.len());
          assert!(actual.as_ref() == expected.as_bytes());
          assert!(wrapped.map(&Default::default()).is_none());
        });
      }
    });
  }
}
