use rspack_cacheable::{enable_cacheable as cacheable, from_bytes, to_bytes, with::AsPreset};
use rspack_sources::{
  BoxSource, CachedSource, ConcatSource, MapOptions, OriginalSource, RawBufferSource,
  RawStringSource, ReplaceSource, Source, SourceExt, SourceMap, SourceMapSource,
  WithoutOriginalOptions,
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

const PROXY_TEXT: &str = "export * from './template';\n";
const PROXY_NAME: &str = "webpack://proxy.vue?vue&type=template&vue3";

fn erased_proxy(replacement: &str) -> BoxSource {
  let leaf = SourceMapSource::new(WithoutOriginalOptions {
    value: PROXY_TEXT.to_owned(),
    name: PROXY_NAME.to_owned(),
    source_map: SourceMap::new(
      "AAAA".into(),
      vec![PROXY_NAME.into()],
      vec![Some(PROXY_TEXT.into())],
      vec![],
    ),
  });
  let mut erased = ReplaceSource::new(leaf.boxed());
  erased.replace(0, PROXY_TEXT.len() as u32, String::new(), None);
  if replacement.is_empty() {
    CachedSource::new(erased).boxed()
  } else {
    // Raw generated text has no mappings, unlike text supplied by ReplaceSource::replace.
    CachedSource::new(ConcatSource::new([
      erased.boxed(),
      RawStringSource::from(replacement.to_owned()).boxed(),
    ]))
    .boxed()
  }
}

fn proxy_asset(proxy: BoxSource, wrapper: &'static str) -> BoxSource {
  ConcatSource::new([
    RawStringSource::from_static(wrapper).boxed(),
    proxy,
    OriginalSource::new("live();\n", "live.js").boxed(),
  ])
  .boxed()
}

fn raw_map(source: &BoxSource, options: &MapOptions) -> String {
  source.map(options).expect("should have a map").to_json()
}

#[test]
fn test_cached_source_fully_deleted_proxy_keeps_first_stream_metadata() {
  for columns in [true, false] {
    for replacement in ["", "/* generated */\n"] {
      let options = MapOptions::new(columns);
      let proxy = erased_proxy(replacement);
      let asset = proxy_asset(proxy.clone(), "// A\n");
      let first = raw_map(&asset, &options);
      assert!(first.contains(PROXY_NAME));
      assert_eq!(proxy.map(&options), None);
      let future = raw_map(&asset, &options);
      assert_eq!(first, future);
      assert_eq!(
        asset.buffer(),
        format!("// A\n{replacement}live();\n").as_bytes()
      );
    }
  }
}

#[test]
fn test_cached_source_shared_proxy_opposite_asset_traversal() {
  for columns in [true, false] {
    let options = MapOptions::new(columns);
    let proxy = erased_proxy("");
    let a = proxy_asset(proxy.clone(), "// A\n");
    let b = proxy_asset(proxy, "// B\n");
    let a_first = raw_map(&a, &options);
    let b_second = raw_map(&b, &options);
    let proxy = erased_proxy("");
    let a_reverse = proxy_asset(proxy.clone(), "// A\n");
    let b_reverse = proxy_asset(proxy, "// B\n");
    let b_first = raw_map(&b_reverse, &options);
    let a_second = raw_map(&a_reverse, &options);
    assert_eq!(a_first, a_second);
    assert_eq!(b_first, b_second);
    assert_eq!(a.buffer(), a_reverse.buffer());
    assert_eq!(b.buffer(), b_reverse.buffer());
  }
}

#[test]
fn test_cached_source_prewarmed_none_and_serialization_preserve_parent_raw_map() {
  for columns in [true, false] {
    let options = MapOptions::new(columns);
    let proxy = erased_proxy("");
    let fresh = raw_map(&proxy_asset(proxy.clone(), "// A\n"), &options);
    assert_eq!(proxy.map(&options), None);
    let bytes = to_bytes(&Data(proxy.clone()), &()).unwrap();
    let restored: Data = from_bytes(&bytes, &()).unwrap();
    // Structural equality includes the SourceMapSource identity/map and ordered replacements.
    assert_eq!(proxy.as_ref(), restored.0.as_ref());
    assert_eq!(proxy.buffer(), restored.0.buffer());
    let inner = |source: &BoxSource| {
      source
        .as_any()
        .downcast_ref::<CachedSource>()
        .unwrap()
        .inner()
        .clone()
    };
    let original = inner(&proxy);
    let decoded = inner(&restored.0);
    let original = original.as_any().downcast_ref::<ReplaceSource>().unwrap();
    let decoded = decoded.as_any().downcast_ref::<ReplaceSource>().unwrap();
    assert_eq!(original.replacements(), decoded.replacements());
    let leaf = decoded
      .inner()
      .as_any()
      .downcast_ref::<SourceMapSource>()
      .unwrap();
    assert_eq!(leaf.name(), PROXY_NAME);
    assert_eq!(
      leaf.source_map().to_json(),
      SourceMap::new(
        "AAAA".into(),
        vec![PROXY_NAME.into()],
        vec![Some(PROXY_TEXT.into())],
        vec![]
      )
      .to_json()
    );

    let warmed = raw_map(&proxy_asset(proxy, "// A\n"), &options);
    let roundtrip = raw_map(&proxy_asset(restored.0, "// A\n"), &options);
    assert_eq!(fresh, warmed);
    assert_eq!(warmed, roundtrip);
    let direct = erased_proxy("");
    assert_eq!(direct.map(&options), None);
    assert_eq!(fresh, raw_map(&proxy_asset(direct, "// A\n"), &options));
  }
}

#[test]
fn test_cached_source_none_replays_identical_stream_events() {
  for columns in [true, false] {
    for replacement in ["", "/* generated */\n"] {
      let options = MapOptions::new(columns);
      let proxy = erased_proxy(replacement);
      let stream = || {
        let mut chunks = Vec::new();
        let mut sources = Vec::new();
        let mut names = Vec::new();
        let end = proxy.stream_chunks().stream(
          &options,
          &mut |chunk, mapping| chunks.push((chunk.map(|c| c.as_str().to_owned()), mapping)),
          &mut |index, name, content| {
            sources.push((
              index,
              name.to_string(),
              content.map(|content| content.to_string()),
            ))
          },
          &mut |index, name| names.push((index, name.to_string())),
        );
        (chunks, sources, names, end)
      };
      let first = stream();
      let future = stream();
      assert_eq!(
        first.1,
        vec![(0, PROXY_NAME.to_owned(), Some(PROXY_TEXT.to_owned()))]
      );
      assert_eq!(first, future);
    }
  }
}
