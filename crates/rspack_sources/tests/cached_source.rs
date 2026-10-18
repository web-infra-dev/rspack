use rspack_sources::{
  CachedSource, ConcatSource, MapOptions, OriginalSource, RawStringSource, Source, SourceExt,
};

#[test]
fn released_rope_cache_is_rebuilt_on_demand() {
  let source = CachedSource::new(ConcatSource::new([
    RawStringSource::from("first\n".to_string()).boxed(),
    RawStringSource::from("second\n".to_string()).boxed(),
  ]));

  assert_eq!(source.source().into_string_lossy(), "first\nsecond\n");

  source.release_caches();

  assert_eq!(source.source().into_string_lossy(), "first\nsecond\n");
  let mut chunks = Vec::new();
  source.rope(&mut |chunk| chunks.push(chunk));
  assert_eq!(chunks.concat(), "first\nsecond\n");
}

#[test]
fn released_source_map_cache_is_rebuilt_on_demand() {
  let source = CachedSource::new(OriginalSource::new(
    "if (ready) {\n  run();\n}\n",
    "index.js",
  ));

  let columns = MapOptions::new(true);
  let line_only = MapOptions::new(false);
  let held_columns = source.map(&columns).unwrap();
  let columns_json = held_columns.to_json();
  let line_only_json = source.map(&line_only).unwrap().to_json();

  source.release_caches();

  assert_eq!(held_columns.to_json(), columns_json);
  assert_eq!(source.map(&columns).unwrap().to_json(), columns_json);
  assert_eq!(source.map(&line_only).unwrap().to_json(), line_only_json);
}
