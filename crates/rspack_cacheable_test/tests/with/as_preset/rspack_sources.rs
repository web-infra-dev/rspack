use rspack_cacheable::{enable_cacheable as cacheable, from_bytes, to_bytes, with::AsPreset};
use rspack_sources::{BoxSource, RawBufferSource, RawStringSource, SourceExt};

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
fn bounded_concat_preserves_large_inputs_and_logical_raw_runs() {
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
fn bounded_concat_coalesces_small_fragments_without_large_copies() {
  use std::hash::{Hash, Hasher};

  use rspack_sources::{ConcatSource, Source};

  let part = "é".repeat(1024);
  let source = ConcatSource::new((0..100).map(|_| RawStringSource::from(part.clone())));
  let expected = part.repeat(100);
  assert_eq!(source.size(), expected.len());
  assert_eq!(source.children().len(), 4);
  assert!(
    source
      .children()
      .iter()
      .all(|child| child.size() <= 64 * 1024)
  );
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
