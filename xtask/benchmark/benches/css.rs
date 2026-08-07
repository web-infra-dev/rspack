use css_module_lexer::{Mode, collect_dependencies};
use rspack_benchmark::{
  BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main,
};

const CSS_CASES: [(&str, &str); 2] = [
  ("bootstrap", include_str!("fixtures/css/bootstrap.css")),
  ("tailwind", include_str!("fixtures/css/tailwind.css")),
];

fn css_module_lexer_benchmark(c: &mut Criterion) {
  let mut group = c.benchmark_group("css_module_lexer");

  for (name, source) in CSS_CASES {
    group.throughput(Throughput::Bytes(source.len() as u64));
    group.bench_with_input(BenchmarkId::from_parameter(name), source, |b, source| {
      b.iter(|| {
        black_box(collect_dependencies(black_box(source), Mode::Local));
      });
    });
  }

  group.finish();
}

criterion_group!(css_benches, css_module_lexer_benchmark);
criterion_main!(css_benches);
