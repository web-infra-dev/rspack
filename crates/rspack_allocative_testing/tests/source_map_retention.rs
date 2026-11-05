use std::sync::Arc;

use rspack_sources::{
  BoxSource, CachedSource, MapOptions, MappingSource, Source, SourceMap, SourceMapSource,
  SourceMapSourceOptions, WithoutOriginalOptions, stream_chunks::StreamChunks,
};

#[cfg(allocative)]
#[test]
fn combined_map_does_not_retain_unused_outer_contents() {
  use rspack_allocative::{FlameGraphBuilder, Visit};

  let source = Arc::new(SourceMapSource::new(SourceMapSourceOptions {
    value: "generated();\n",
    name: "intermediate.js",
    source_map: SourceMap::new(
      "AAAAA",
      vec!["intermediate.js".into()],
      vec!["unused intermediate text".repeat(400_000).into()],
      vec!["outerName".into()],
    ),
    original_source: None,
    inner_source_map: Some(SourceMap::new(
      "AAAA",
      vec!["original.js".into()],
      vec!["original();\n".into()],
      vec![],
    )),
    remove_original_source: true,
  }));
  let expected = source.map(&MapOptions::default()).unwrap().to_json();
  let map = source.map_static(&MapOptions::default()).unwrap();
  assert_eq!(map.to_json(), expected);
  let mut builder = FlameGraphBuilder::with_shared_ownership();
  let mut visitor = builder.root_visitor();
  map.visit_memory(&mut visitor);
  visitor.exit();
  let profile = builder.finish();
  assert!(profile.warnings().is_empty(), "{}", profile.warnings());
  assert!(
    profile.flamegraph().total_size() < 1024 * 1024,
    "the composed map must not retain the unused outer source contents"
  );
}

#[test]
fn external_map_releases_generated_source_without_changing_mappings() {
  for columns in [true, false] {
    let source = Arc::new(SourceMapSource::new(WithoutOriginalOptions {
      value: "unmapped generated text\n".repeat(100_000),
      name: "generated.js",
      source_map: SourceMap::new(
        "AAAA",
        vec!["original-π.js".into()],
        vec!["console.log('💡');\n".into()],
        vec![],
      ),
    }));
    let weak = Arc::downgrade(&source);
    let map = source
      .clone()
      .map_static(&MapOptions::new(columns))
      .unwrap();
    let expected = map.to_json();
    let asset = MappingSource::new(map);
    drop(source);

    assert!(
      weak.upgrade().is_none(),
      "an external map must not retain its generated source graph"
    );
    assert_eq!(asset.source().into_string_lossy(), expected);
    assert_eq!(asset.size(), expected.len());
    let mut written = Vec::new();
    asset.to_writer(&mut written).unwrap();
    assert_eq!(written, expected.as_bytes());
    let reparsed = SourceMap::from_json(expected).unwrap();
    assert_eq!(reparsed.get_source(0), Some("original-π.js"));
    assert_eq!(
      reparsed.get_source_content(0).map(|s| s.as_ref()),
      Some("console.log('💡');\n")
    );
  }
}

#[test]
fn combined_map_shares_inner_contents_and_preserves_outer_sources() {
  let inner = SourceMap::new(
    "AAAA",
    vec!["original.js".into()],
    vec!["original();\n".repeat(100_000).into()],
    vec![],
  );
  let inner_content = inner.get_source_content(0).unwrap().as_ptr();
  let outer = SourceMap::from_json(
    r#"{"version":3,"sources":["intermediate.js","external-π.js"],"sourcesContent":["intermediate();","external();"],"names":["innerName","outerName"],"mappings":"AAAAA;ACAAC"}"#
      .to_string(),
  )
  .unwrap();
  let outer_content = outer.get_source_content(1).unwrap().as_ptr();
  let source = Arc::new(SourceMapSource::new(SourceMapSourceOptions {
    value: "first();\nsecond();\n",
    name: "intermediate.js",
    source_map: outer,
    original_source: None,
    inner_source_map: Some(inner),
    remove_original_source: true,
  }));
  let options = MapOptions::default();
  let expected = source.map(&options).unwrap().to_json();
  let map = source.map_static(&options).unwrap();

  assert_eq!(map.to_json(), expected);
  let inner_index = map
    .sources()
    .iter()
    .position(|s| s == "original.js")
    .unwrap();
  let outer_index = map
    .sources()
    .iter()
    .position(|s| s == "external-π.js")
    .unwrap();
  assert_eq!(
    map.get_source_content(inner_index).unwrap().as_ptr(),
    inner_content
  );
  assert_eq!(
    map.get_source_content(outer_index).unwrap().as_ref(),
    "external();"
  );
  assert_ne!(
    map.get_source_content(outer_index).unwrap().as_ptr(),
    outer_content
  );
  assert!(map.names().iter().any(|name| name == "outerName"));
}

#[test]
fn combined_map_keeps_only_the_source_contents_it_emits() {
  for remove_original_source in [false, true] {
    let original = "intermediate();\nunmapped();\n";
    let source = Arc::new(SourceMapSource::new(SourceMapSourceOptions {
      value: "first();\nsecond();\n",
      name: "intermediate.js",
      source_map: SourceMap::new("AAAA;AACA", vec!["intermediate.js".into()], vec![], vec![]),
      original_source: Some(original.into()),
      inner_source_map: Some(SourceMap::new(
        "AAAA",
        vec!["original.js".into()],
        vec!["original();\n".into()],
        vec![],
      )),
      remove_original_source,
    }));
    let options = MapOptions::default();
    let expected = source.map(&options).unwrap().to_json();
    let weak = Arc::downgrade(&source);
    let map = source.clone().map_static(&options).unwrap();
    drop(source);

    assert!(weak.upgrade().is_none());
    assert_eq!(map.to_json(), expected);
    assert_eq!(
      map.sources_content().iter().any(|s| s.as_ref() == original),
      !remove_original_source
    );
  }
}

#[test]
fn external_map_outlives_a_shared_cached_source() {
  for stream_first in [false, true] {
    let original = "console.log('π💡');\n".repeat(10_000);
    let inner = Arc::new(SourceMapSource::new(WithoutOriginalOptions {
      value: "generated();\n".repeat(100_000),
      name: "generated.js",
      source_map: SourceMap::new(
        "AAAA",
        vec!["original.js".into()],
        vec![original.clone().into()],
        vec![],
      ),
    }));
    let weak_inner = Arc::downgrade(&inner);
    let source = Arc::new(CachedSource::new(inner as BoxSource));
    let weak = Arc::downgrade(&source);
    let options = MapOptions::new(true);
    if stream_first {
      source
        .stream_chunks()
        .stream(&options, &mut |_, _| {}, &mut |_, _, _| {}, &mut |_, _| {});
    }
    let map = source.clone().map_static(&options).unwrap();
    let expected = map.to_json();
    let asset = MappingSource::new(map);

    assert_eq!(source.map(&options).unwrap().to_json(), expected);
    assert_eq!(asset.source().into_string_lossy(), expected);
    drop(source);
    assert!(weak.upgrade().is_none());
    assert!(weak_inner.upgrade().is_none());
    assert_eq!(asset.source().into_string_lossy(), expected);
    let reparsed = SourceMap::from_json(asset.source().into_string_lossy().into_owned()).unwrap();
    assert_eq!(reparsed.get_source(0), Some("original.js"));
    assert_eq!(reparsed.get_source_content(0).unwrap().as_ref(), original);
  }
}

#[test]
fn external_map_preserves_borrowed_metadata_and_empty_fields() {
  let map = SourceMap::from_json(
    r#"{"version":3,"file":"","sources":["original-💡.js",""],"sourcesContent":["let π = '\"💡\"';\n",""],"names":["π",""],"mappings":"AAAAA","sourceRoot":"","debugId":"","ignoreList":[]}"#
      .to_string(),
  )
  .unwrap();
  let expected = map.to_json();
  let expected_mappings = map.mappings().to_string();
  let asset = MappingSource::new(map);

  assert_eq!(asset.source().into_string_lossy(), expected);
  assert_eq!(asset.buffer().as_ref(), expected.as_bytes());
  let reparsed = SourceMap::from_json(asset.source().into_string_lossy().into_owned()).unwrap();
  assert_eq!(reparsed.file(), Some(""));
  assert_eq!(reparsed.source_root(), Some(""));
  assert_eq!(reparsed.get_debug_id(), Some(""));
  assert_eq!(reparsed.ignore_list(), Some([].as_slice()));
  assert_eq!(reparsed.get_name(0), Some("π"));
  assert_eq!(reparsed.get_name(1), Some(""));
  assert_eq!(reparsed.mappings(), expected_mappings);
}
