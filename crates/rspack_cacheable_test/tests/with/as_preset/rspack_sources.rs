use rspack_cacheable::{enable_cacheable as cacheable, from_bytes, to_bytes, with::AsPreset};
use rspack_sources::{
  BoxSource, CachedSource, ConcatSource, ObjectPool, OriginalSource, RawBufferSource,
  RawStringSource, ReplaceSource, ReplacementEnforce, SourceExt, SourceMap, SourceMapSource,
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
      data.0.map(&ObjectPool::default(), &Default::default()),
      new_data.0.map(&ObjectPool::default(), &Default::default())
    );
  }

  test_data(Data(RawBufferSource::from("123".as_bytes()).boxed()));
  test_data(Data(RawStringSource::from_static("123").boxed()));
  test_data(Data(
    OriginalSource::new("const answer = 42;\n", "answer.js").boxed(),
  ));
  test_data(Data(
    SourceMapSource::new(WithoutOriginalOptions {
      value: "const answer = 42;\n",
      name: "answer.js",
      source_map: SourceMap::from_json(
        r#"{
          "version": 3,
          "sources": ["answer.ts"],
          "sourcesContent": ["const answer: number = 42;\n"],
          "names": [],
          "mappings": "AAAA"
        }"#,
      )
      .unwrap(),
    })
    .boxed(),
  ));
  test_data(Data(
    ConcatSource::new([
      RawStringSource::from_static("const ").boxed(),
      OriginalSource::new("answer = 42;\n", "answer.js").boxed(),
    ])
    .boxed(),
  ));

  let mut replace_source = ReplaceSource::new(OriginalSource::new("hello world\n", "hello.txt"));
  replace_source.replace_with_enforce(
    6,
    11,
    "rspack".to_string(),
    Some("tool".to_string()),
    ReplacementEnforce::Post,
  );
  test_data(Data(replace_source.boxed()));

  test_data(Data(
    CachedSource::new(ConcatSource::new([
      RawStringSource::from_static("cached ").boxed(),
      RawStringSource::from_static("source").boxed(),
    ]))
    .boxed(),
  ));
}

#[test]
fn test_concat_source_serializes_optimized_children() {
  let data = Data(
    ConcatSource::new([
      RawStringSource::from_static("cached ").boxed(),
      RawStringSource::from_static("source").boxed(),
    ])
    .boxed(),
  );

  let bytes = to_bytes(&data, &()).unwrap();
  let new_data: Data = from_bytes(&bytes, &()).unwrap();
  let concat_source = new_data
    .0
    .as_ref()
    .as_any()
    .downcast_ref::<ConcatSource>()
    .unwrap();
  let debug = format!("{concat_source:?}");

  assert_eq!(debug.matches("RawStringSource::from_static").count(), 1);
  assert!(debug.contains("\"cached source\""));
}
