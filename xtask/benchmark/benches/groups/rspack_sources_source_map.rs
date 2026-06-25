#![allow(missing_docs)]
#![allow(clippy::unwrap_used)]

use criterion::Bencher;
use rspack_sources::SourceMap;

const ANTD_MIN_JS_MAP: &str = include_str!(concat!(
  env!("CARGO_MANIFEST_DIR"),
  "/benches/fixtures/rspack_sources/antd-mini/antd.min.js.map"
));

pub fn benchmark_parse_source_map_from_json(b: &mut Bencher) {
  b.iter(|| std::hint::black_box(SourceMap::from_json(ANTD_MIN_JS_MAP.to_string()).unwrap()))
}

pub fn benchmark_source_map_to_json(b: &mut Bencher) {
  let source_map = SourceMap::from_json(ANTD_MIN_JS_MAP.to_string()).unwrap();
  b.iter(|| source_map.to_json())
}
