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
fn test_source_map_contents_roundtrip() {
  use rspack_sources::{
    CachedSource, ConcatSource, MappingSource, Source, SourceMap, SourceMapSource,
    WithoutOriginalOptions,
  };

  // A duplicate known key exercises the DOM fallback as well as the tape parser.
  for duplicate in ["", "\"names\":[],"] {
    for contents in [
      r#"[null,"text\n","",null]"#,
      r#"["","","",""]"#,
      "[null,null,null,null]",
    ] {
      let json = format!(
        r#"{{{duplicate}"version":3,"sources":["a.js","b.js","c.js","d.js"],"sourcesContent":{contents},"names":[],"mappings":"AAAA;ACAA;ACAA;ACAA"}}"#
      );
      let map = SourceMap::from_json(json.clone()).unwrap();
      let expected: Vec<Option<String>> = serde_json::from_str(contents).unwrap();
      assert_eq!(
        map
          .sources_content()
          .iter()
          .map(Option::as_deref)
          .collect::<Vec<_>>(),
        expected.iter().map(Option::as_deref).collect::<Vec<_>>()
      );

      let mapping_source = MappingSource::new(map);
      let mut rope = String::new();
      mapping_source.rope(&mut |chunk| rope.push_str(chunk));
      assert_eq!(rope.as_bytes(), mapping_source.buffer().as_ref());
      let data = Data(mapping_source.boxed());
      let restored: Data = from_bytes(&to_bytes(&data, &()).unwrap(), &()).unwrap();
      assert_eq!(data.0.buffer(), restored.0.buffer());
      let mapping_source = restored.0.as_any().downcast_ref::<MappingSource>().unwrap();
      assert_eq!(
        mapping_source.sources_content().collect::<Vec<_>>(),
        expected
          .iter()
          .filter_map(Option::as_deref)
          .collect::<Vec<_>>()
      );

      // Force map reconstruction, including CachedSource's stream-first path.
      let source = CachedSource::new(SourceMapSource::new(WithoutOriginalOptions {
        value: "a();\nb();\nc();\nd();\n",
        name: "generated.js",
        source_map: SourceMap::from_json(json).unwrap(),
      }));
      let source = ConcatSource::new([source.boxed()]);
      for source in [source.boxed(), restored.0] {
        let data = Data(source);
        let restored: Data = from_bytes(&to_bytes(&data, &()).unwrap(), &()).unwrap();
        for source in [&data.0, &restored.0] {
          let json = source.map(&Default::default()).map_or_else(
            || source.source().into_string_lossy().into_owned(),
            |map| map.to_json(),
          );
          let map: serde_json::Value = serde_json::from_str(&json).unwrap();
          if expected.iter().all(Option::is_none) {
            assert!(map.get("sourcesContent").is_none());
          } else {
            assert_eq!(
              map["sourcesContent"],
              serde_json::to_value(&expected).unwrap()
            );
          }
        }
      }
    }
  }
}
