use std::sync::Arc;

use criterion::black_box;
use rspack_benchmark::Criterion;
use rspack_javascript_compiler::benchmark_source_map_position_conversion;
use rspack_sources::SourceMap;
use swc_core::common::{BytePos, FileName, SourceFile, SourceMap as SwcSourceMap};

const ANTD_MIN_JS: &str = include_str!(concat!(
  env!("CARGO_MANIFEST_DIR"),
  "/benches/fixtures/rspack_sources/antd-mini/antd.min.js"
));
const ANTD_MIN_JS_MAP: &str = include_str!(concat!(
  env!("CARGO_MANIFEST_DIR"),
  "/benches/fixtures/rspack_sources/antd-mini/antd.min.js.map"
));

struct SourceMapPositionBenchmark {
  file: Arc<SourceFile>,
  positions: Vec<BytePos>,
}

impl SourceMapPositionBenchmark {
  fn new() -> Self {
    let line_starts = std::iter::once(0)
      .chain(ANTD_MIN_JS.match_indices('\n').map(|(index, _)| index + 1))
      .collect::<Vec<_>>();
    let mappings = SourceMap::from_json(ANTD_MIN_JS_MAP.to_string())
      .expect("antd source map fixture should be valid")
      .decoded_mappings()
      .map(|mapping| {
        (
          usize::try_from(mapping.generated_line - 1).expect("generated line should fit usize"),
          mapping.generated_column,
        )
      })
      .collect::<Vec<_>>();
    let mut positions = Vec::with_capacity(mappings.len());
    let mut current_line = usize::MAX;
    let mut current_utf16_column = 0;
    let mut current_byte_column = 0;
    for (line, utf16_column) in mappings {
      if line != current_line {
        current_line = line;
        current_utf16_column = 0;
        current_byte_column = 0;
      }
      assert!(
        current_utf16_column <= utf16_column,
        "source map mappings should be ordered"
      );

      let line_start = line_starts[line];
      let line_end = line_starts
        .get(line + 1)
        .map_or(ANTD_MIN_JS.len(), |next_line_start| next_line_start - 1);
      for character in ANTD_MIN_JS[line_start + current_byte_column..line_end].chars() {
        if current_utf16_column == utf16_column {
          break;
        }
        current_utf16_column += character.len_utf16() as u32;
        current_byte_column += character.len_utf8();
      }
      assert_eq!(
        current_utf16_column, utf16_column,
        "source map column should be a UTF-16 character boundary"
      );
      positions.push(line_start + current_byte_column);
    }

    let source_map = SwcSourceMap::default();
    let file = source_map.new_source_file(
      Arc::new(FileName::Custom("antd.min.js".into())),
      ANTD_MIN_JS,
    );
    let positions = positions
      .into_iter()
      .map(|position| {
        file.start_pos + BytePos(u32::try_from(position).expect("fixture position should fit u32"))
      })
      .collect();

    Self { file, positions }
  }

  fn run(&self) -> u64 {
    benchmark_source_map_position_conversion(&self.file, &self.positions)
  }
}

pub fn benchmark_javascript_compiler_source_map(c: &mut Criterion) {
  let benchmark = SourceMapPositionBenchmark::new();
  assert!(!benchmark.positions.is_empty());
  assert_ne!(benchmark.run(), 0);

  c.bench_function(
    "rust@javascript_compiler_source_map@antd_minified_positions",
    |b| b.iter(|| black_box(benchmark.run())),
  );
}
