use rspack_cacheable::{enable_cacheable as cacheable, from_bytes, to_bytes, with::AsPreset};
use rspack_sources::{
  BoxSource, OriginalSource, RawBufferSource, RawStringSource, SourceExt, SourceOnlySource,
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
    assert!(data.0.as_ref().eq(new_data.0.as_ref()));
    assert_eq!(
      data.0.map(&Default::default()),
      new_data.0.map(&Default::default())
    );
  }

  test_data(Data(RawBufferSource::from("123".as_bytes()).boxed()));
  test_data(Data(RawStringSource::from_static("123").boxed()));
  test_data(Data(
    SourceOnlySource::new(OriginalSource::new("hello 😀", "original.js").boxed()).boxed(),
  ));
  test_data(Data(
    SourceOnlySource::new(RawBufferSource::from(vec![0xff, 0, 0x61]).boxed()).boxed(),
  ));
}

#[test]
fn concat_preserves_large_inputs_and_logical_raw_runs() {
  use std::hash::{Hash, Hasher};

  use rspack_sources::{ConcatSource, OriginalSource, Source};

  fn hash(source: &BoxSource) -> u64 {
    let mut hasher = rustc_hash::FxHasher::default();
    source.hash(&mut hasher);
    hasher.finish()
  }

  let large = RawStringSource::from("a😀\r\n".repeat(40_000)).boxed();
  let suffix = RawStringSource::from_static("//# sourceMappingURL=bundle.js.map\n").boxed();
  let mut concat = ConcatSource::new([large.clone(), suffix.clone()]);
  let expected = format!(
    "{}{}",
    large.source().into_string_lossy(),
    suffix.source().into_string_lossy()
  );
  assert_eq!(concat.size(), expected.len());
  assert!(std::sync::Arc::ptr_eq(&concat.children()[0], &large));
  assert!(std::sync::Arc::ptr_eq(&concat.children()[1], &suffix));
  let mut written = Vec::new();
  concat.to_writer(&mut written).unwrap();
  assert_eq!(written, expected.as_bytes());
  assert_eq!(concat.buffer().as_ref(), expected.as_bytes());
  assert_eq!(concat.source().into_string_lossy(), expected);

  concat.add(RawStringSource::from_static("tail"));
  let split = concat.boxed();
  let flat = ConcatSource::new([RawStringSource::from(format!("{expected}tail"))]).boxed();
  assert_eq!(split.as_ref(), flat.as_ref());
  assert_eq!(hash(&split), hash(&flat));
  let restored: Data = from_bytes(&to_bytes(&Data(split.clone()), &()).unwrap(), &()).unwrap();
  assert_eq!(split.as_ref(), restored.0.as_ref());
  assert_eq!(hash(&split), hash(&restored.0));

  // Raw boundaries must not introduce extra mappings between mapped sections.
  let before = OriginalSource::new("const before = 1;\n", "before.js").boxed();
  let after = OriginalSource::new("const after = 2;\n", "after.js").boxed();
  let mapped_split = ConcatSource::new([before.clone(), split, after.clone()]);
  let mapped_flat = ConcatSource::new([before, flat, after]);
  for columns in [false, true] {
    let options = rspack_sources::MapOptions::new(columns);
    assert_eq!(mapped_split.map(&options), mapped_flat.map(&options));
  }
}

#[test]
fn concat_still_coalesces_small_fragments() {
  use std::hash::{Hash, Hasher};

  use rspack_sources::{ConcatSource, Source};

  let part = "é".repeat(1024);
  let mut source = ConcatSource::new((0..100).map(|_| RawStringSource::from(part.clone())));
  let expected = part.repeat(100);
  assert_eq!(source.size(), expected.len());
  assert_eq!(source.children().len(), 1);
  // A merged run can exceed the threshold and must survive later normalization.
  let merged = source.children()[0].clone();
  source.add(RawStringSource::from_static(""));
  assert_eq!(source.children().len(), 2);
  assert!(std::sync::Arc::ptr_eq(&source.children()[0], &merged));
  let split = source.boxed();
  let flat = ConcatSource::new([RawStringSource::from(expected)]).boxed();
  assert_eq!(split.as_ref(), flat.as_ref());
  let mut a = std::collections::hash_map::DefaultHasher::new();
  let mut b = std::collections::hash_map::DefaultHasher::new();
  split.hash(&mut a);
  flat.hash(&mut b);
  assert_eq!(a.finish(), b.finish());

  let different = ConcatSource::new([RawStringSource::from(part.repeat(99))]).boxed();
  assert_ne!(split.as_ref(), different.as_ref());
  let same_size = ConcatSource::new([RawStringSource::from("ê".repeat(102_400))]).boxed();
  assert_eq!(split.size(), same_size.size());
  assert_ne!(split.as_ref(), same_size.as_ref());
  let empty = ConcatSource::new([
    RawStringSource::from_static(""),
    RawStringSource::from_static(""),
  ])
  .boxed();
  assert_ne!(split.as_ref(), empty.as_ref());
}

#[test]
fn concat_preserves_maps_through_replacements() {
  use rspack_sources::{ConcatSource, MapOptions, OriginalSource, ReplaceSource, Source};

  for prefix in ["é".repeat(40_000), "😀".repeat(20_000)] {
    for suffix in ["é😀tail", "\r\né😀tail"] {
      let split = ConcatSource::new([
        RawStringSource::from(prefix.clone()).boxed(),
        RawStringSource::from_static("").boxed(),
        RawStringSource::from(suffix).boxed(),
        OriginalSource::new("after", "after.js").boxed(),
      ])
      .boxed();
      let flat = ConcatSource::new([
        RawStringSource::from(format!("{prefix}{suffix}")).boxed(),
        OriginalSource::new("after", "after.js").boxed(),
      ])
      .boxed();
      let mut split = ReplaceSource::new(split);
      let mut flat = ReplaceSource::new(flat);
      split.insert_static(prefix.len() as u32, "X😀\n", None);
      flat.insert_static(prefix.len() as u32, "X😀\n", None);
      assert_eq!(split.source(), flat.source());
      for columns in [false, true] {
        let options = MapOptions::new(columns);
        let map = split.map(&options).unwrap();
        assert_eq!(Some(&map), flat.map(&options).as_ref());
        if columns {
          let after = map
            .decoded_mappings()
            .find(|mapping| mapping.original.is_some())
            .unwrap();
          assert_eq!(
            after.generated_line,
            if suffix.starts_with('\r') { 3 } else { 2 }
          );
          assert_eq!(after.generated_column, 7);
        }
      }
    }
  }
}

#[test]
fn source_only_preserves_materialized_bytes() {
  use std::sync::Arc;

  use rspack_sources::{CachedSource, ConcatSource, MapOptions, ReplaceSource, SourceValue};

  let binary = RawBufferSource::from(vec![0xff, 0, b'x']).boxed();
  let mixed = ConcatSource::new([
    OriginalSource::new("// mapped\n", "original.js").boxed(),
    Arc::clone(&binary),
  ])
  .boxed();
  assert!(mixed.map(&MapOptions::default()).is_some());

  let sources = [
    Arc::clone(&binary),
    Arc::clone(&mixed),
    CachedSource::new(Arc::clone(&mixed)).boxed(),
    ReplaceSource::new(Arc::clone(&mixed)).boxed(),
    ReplaceSource::new(Arc::clone(&binary)).boxed(),
    ConcatSource::new([Arc::clone(&binary)]).boxed(),
  ];
  for inner in sources {
    let expected = inner.source();
    let is_binary = matches!(expected, SourceValue::Buffer(_));
    let expected = expected.into_bytes().into_owned();
    let source = SourceOnlySource::new(inner).boxed();
    let bytes = to_bytes(&Data(Arc::clone(&source)), &()).unwrap();
    let restored: Data = from_bytes(&bytes, &()).unwrap();
    for source in [source, restored.0] {
      assert_eq!(matches!(source.source(), SourceValue::Buffer(_)), is_binary);
      assert_eq!(source.source().into_bytes().as_ref(), expected);
      assert_eq!(source.buffer().as_ref(), expected);
      let mut written = Vec::new();
      source.to_writer(&mut written).unwrap();
      assert_eq!(written, expected);
      assert_eq!(source.size(), expected.len());
      assert!(source.map(&MapOptions::default()).is_none());
    }
  }

  // Ordinary UTF-8 graphs stay shared rather than being flattened.
  let text = ConcatSource::new([
    OriginalSource::new("// mapped\n", "original.js").boxed(),
    RawBufferSource::from("😀".as_bytes()).boxed(),
  ])
  .boxed();
  let source = SourceOnlySource::new(Arc::clone(&text));
  assert!(Arc::ptr_eq(source.inner(), &text));
}
