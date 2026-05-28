#![allow(missing_docs)]
#![allow(clippy::zero_sized_map_values)]

#[path = "rspack_sources_complex_replace_source.rs"]
mod bench_complex_replace_source;
#[path = "rspack_sources_source_map.rs"]
mod bench_source_map;
#[path = "rspack_sources_repetitive_react_components.rs"]
mod benchmark_repetitive_react_components;

use std::collections::HashMap;

use bench_complex_replace_source::{
  benchmark_complex_replace_source_map,
  benchmark_complex_replace_source_map_cached_source_stream_chunks,
  benchmark_complex_replace_source_size, benchmark_complex_replace_source_source,
};
use bench_source_map::{benchmark_parse_source_map_from_json, benchmark_source_map_to_json};
use benchmark_repetitive_react_components::{
  benchmark_repetitive_react_components_map, benchmark_repetitive_react_components_source,
};
use criterion::Bencher;
use rspack_benchmark::Criterion;
use rspack_sources::{
  BoxSource, CachedSource, ConcatSource, MapOptions, ObjectPool, RawStringSource, Source,
  SourceExt, SourceMap, SourceMapSource, SourceMapSourceOptions,
};

const HELLOWORLD_JS: &str = include_str!(concat!(
  env!("CARGO_MANIFEST_DIR"),
  "/benches/fixtures/rspack_sources/transpile-minify/files/helloworld.js"
));
const HELLOWORLD_JS_MAP: &str = include_str!(concat!(
  env!("CARGO_MANIFEST_DIR"),
  "/benches/fixtures/rspack_sources/transpile-minify/files/helloworld.js.map"
));
const HELLOWORLD_MIN_JS: &str = include_str!(concat!(
  env!("CARGO_MANIFEST_DIR"),
  "/benches/fixtures/rspack_sources/transpile-minify/files/helloworld.min.js"
));
const HELLOWORLD_MIN_JS_MAP: &str = include_str!(concat!(
  env!("CARGO_MANIFEST_DIR"),
  "/benches/fixtures/rspack_sources/transpile-minify/files/helloworld.min.js.map"
));
const BUNDLE_JS: &str = include_str!(concat!(
  env!("CARGO_MANIFEST_DIR"),
  "/benches/fixtures/rspack_sources/transpile-rollup/files/bundle.js"
));
const BUNDLE_JS_MAP: &str = include_str!(concat!(
  env!("CARGO_MANIFEST_DIR"),
  "/benches/fixtures/rspack_sources/transpile-rollup/files/bundle.js.map"
));

fn benchmark_concat_generate_string(b: &mut Bencher) {
  let sms_minify = SourceMapSource::new(SourceMapSourceOptions {
    value: HELLOWORLD_MIN_JS,
    name: "helloworld.min.js",
    source_map: SourceMap::from_json(HELLOWORLD_MIN_JS_MAP).unwrap(),
    original_source: Some(HELLOWORLD_JS.to_string().into()),
    inner_source_map: Some(SourceMap::from_json(HELLOWORLD_JS_MAP).unwrap()),
    remove_original_source: false,
  });

  let sms_rollup = SourceMapSource::new(SourceMapSourceOptions {
    value: BUNDLE_JS,
    name: "bundle.js",
    source_map: SourceMap::from_json(BUNDLE_JS_MAP).unwrap(),
    original_source: None,
    inner_source_map: None,
    remove_original_source: false,
  });

  let concat = ConcatSource::new([sms_minify, sms_rollup]);

  b.iter(|| {
    concat
      .map(&ObjectPool::default(), &MapOptions::default())
      .unwrap()
      .to_json();
  })
}

fn benchmark_concat_generate_string_with_cache(b: &mut Bencher) {
  let sms_minify = SourceMapSource::new(SourceMapSourceOptions {
    value: HELLOWORLD_MIN_JS,
    name: "helloworld.min.js",
    source_map: SourceMap::from_json(HELLOWORLD_MIN_JS_MAP).unwrap(),
    original_source: Some(HELLOWORLD_JS.to_string().into()),
    inner_source_map: Some(SourceMap::from_json(HELLOWORLD_JS_MAP).unwrap()),
    remove_original_source: false,
  });
  let sms_rollup = SourceMapSource::new(SourceMapSourceOptions {
    value: BUNDLE_JS,
    name: "bundle.js",
    source_map: SourceMap::from_json(BUNDLE_JS_MAP).unwrap(),
    original_source: None,
    inner_source_map: None,
    remove_original_source: false,
  });
  let concat = ConcatSource::new([sms_minify, sms_rollup]);
  let cached = CachedSource::new(concat);

  b.iter(|| {
    cached
      .map(&ObjectPool::default(), &MapOptions::default())
      .unwrap()
      .to_json();
  })
}

fn benchmark_cached_source_hash(b: &mut Bencher) {
  let sms_minify = SourceMapSource::new(SourceMapSourceOptions {
    value: HELLOWORLD_MIN_JS,
    name: "helloworld.min.js",
    source_map: SourceMap::from_json(HELLOWORLD_MIN_JS_MAP).unwrap(),
    original_source: Some(HELLOWORLD_JS.to_string().into()),
    inner_source_map: Some(SourceMap::from_json(HELLOWORLD_JS_MAP).unwrap()),
    remove_original_source: false,
  });
  let sms_rollup = SourceMapSource::new(SourceMapSourceOptions {
    value: BUNDLE_JS,
    name: "bundle.js",
    source_map: SourceMap::from_json(BUNDLE_JS_MAP).unwrap(),
    original_source: None,
    inner_source_map: None,
    remove_original_source: false,
  });
  let concat = ConcatSource::new([sms_minify, sms_rollup]);
  let cached = CachedSource::new(concat).boxed();

  b.iter(|| {
    let mut m = HashMap::<BoxSource, ()>::new();
    m.insert(cached.clone(), ());
    let _ = std::hint::black_box(|| m.get(&cached));
    let _ = std::hint::black_box(|| m.get(&cached));
  })
}

fn benchmark_concat_source_add_many(b: &mut Bencher) {
  // Mimic rspack's concatenated_module / runtime hot path: build a ConcatSource
  // by adding many small children sequentially. 500 matches the scale of a
  // typical concatenated module (rspack chains 300+ adds per module).
  let pieces: Vec<BoxSource> = (0..500)
    .map(|i| RawStringSource::from(format!("// piece {i}\n")).boxed())
    .collect();

  b.iter(|| {
    let mut concat = ConcatSource::default();
    for piece in &pieces {
      concat.add(piece.clone());
    }
    std::hint::black_box(concat);
  })
}

fn benchmark_concat_source_add_few(b: &mut Bencher) {
  // Smaller scale: closer to runtime module assembly (~10-15 adds per chunk).
  let pieces: Vec<BoxSource> = (0..16)
    .map(|i| RawStringSource::from(format!("// piece {i}\n")).boxed())
    .collect();

  b.iter(|| {
    let mut concat = ConcatSource::default();
    for piece in &pieces {
      concat.add(piece.clone());
    }
    std::hint::black_box(concat);
  })
}

fn bench_rspack_sources(criterion: &mut Criterion) {
  let mut group = criterion.benchmark_group("rspack_sources");

  group.bench_function(
    "concat_generate_string_with_cache",
    benchmark_concat_generate_string_with_cache,
  );
  group.bench_function("concat_generate_string", benchmark_concat_generate_string);

  group.bench_function("cached_source_hash", benchmark_cached_source_hash);

  group.bench_function("concat_source_add_many", benchmark_concat_source_add_many);
  group.bench_function("concat_source_add_few", benchmark_concat_source_add_few);

  group.bench_function(
    "complex_replace_source_map",
    benchmark_complex_replace_source_map,
  );

  group.bench_function(
    "complex_replace_source_map_cached_source_stream_chunks",
    benchmark_complex_replace_source_map_cached_source_stream_chunks,
  );

  group.bench_function(
    "complex_replace_source_source",
    benchmark_complex_replace_source_source,
  );

  group.bench_function(
    "complex_replace_source_size",
    benchmark_complex_replace_source_size,
  );

  group.bench_function(
    "parse_source_map_from_json",
    benchmark_parse_source_map_from_json,
  );

  group.bench_function("source_map_to_json", benchmark_source_map_to_json);

  group.bench_function(
    "repetitive_react_components_map",
    benchmark_repetitive_react_components_map,
  );

  group.bench_function(
    "repetitive_react_components_source",
    benchmark_repetitive_react_components_source,
  );

  group.finish();
}

pub fn rspack_sources_benchmark(c: &mut Criterion) {
  bench_rspack_sources(c);
}
