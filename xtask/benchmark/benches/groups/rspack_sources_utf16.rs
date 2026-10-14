#![allow(missing_docs)]

use criterion::Bencher;
use rspack_sources::{
  MapOptions, Mapping, OriginalLocation, ReplaceSource, Source, SourceMap, SourceMapSource,
  SourceMapSourceOptions, encode_mappings,
};

fn source_with_mappings(value: String, mapping_step: usize) -> ReplaceSource {
  let utf16_len = value.encode_utf16().count();
  let mappings = encode_mappings((0..utf16_len).step_by(mapping_step).map(|column| Mapping {
    generated_line: 1,
    generated_column: column as u32,
    original: Some(OriginalLocation {
      source_index: 0,
      original_line: 1,
      original_column: column as u32,
      name_index: None,
    }),
  }));
  let source_map = SourceMap::from_json(format!(
    r#"{{"version":3,"sources":["input.js"],"names":[],"mappings":"{mappings}"}}"#
  ))
  .unwrap();
  let mut source = ReplaceSource::new(SourceMapSource::new(SourceMapSourceOptions {
    value,
    name: "output.js",
    source_map,
    original_source: None,
    inner_source_map: None,
    remove_original_source: false,
  }));
  source.insert_static(0, "/* marker */", None);
  source
}

pub fn benchmark_sparse_non_ascii_map(b: &mut Bencher) {
  let source = source_with_mappings(format!("{}🦀{}", "a".repeat(32768), "a".repeat(32768)), 32);
  b.iter(|| std::hint::black_box(source.map(&MapOptions::default())));
}

pub fn benchmark_dense_non_ascii_map(b: &mut Bencher) {
  let source = source_with_mappings("界".repeat(8192), 8);
  b.iter(|| std::hint::black_box(source.map(&MapOptions::default())));
}
