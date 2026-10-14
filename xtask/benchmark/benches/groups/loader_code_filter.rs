use criterion::{BenchmarkId, black_box};
use rspack_benchmark::Criterion;
use rspack_loader_runner::{CodeFilter, Content};
use rspack_regex::RspackRegex;

const MODULE_COUNT: usize = 100;
const MODULE_PREFIX: &str = "export const value = 42;\n";

fn modules(hit_percent: usize) -> Vec<Content> {
  (0..MODULE_COUNT)
    .map(|index| {
      let mut code = String::with_capacity(MODULE_PREFIX.len() * 512 + 50);
      if index < hit_percent {
        code.push_str("export const handler = createServerFn();\n");
      }
      for _ in 0..512 {
        code.push_str(MODULE_PREFIX);
      }
      Content::String(code)
    })
    .collect()
}

pub fn benchmark_loader_code_filter(criterion: &mut Criterion) {
  let mut group = criterion.benchmark_group("loader_code_filter");

  for hit_percent in [0, 10, 100] {
    let modules = modules(hit_percent);
    let regexp_filter = CodeFilter::new(
      Some(
        RspackRegex::new(r"\bcreate(ServerFn|Middleware)\b")
          .expect("benchmark regexp should be valid"),
      ),
      None,
    );
    group.bench_with_input(
      BenchmarkId::new("regexp", hit_percent),
      &hit_percent,
      |bencher, _| {
        bencher.iter(|| {
          for content in &modules {
            black_box(regexp_filter.matches(Some(black_box(content))));
          }
        });
      },
    );

    let regexp_with_exclude_filter = CodeFilter::new(
      regexp_filter.include().cloned(),
      Some(RspackRegex::new("generated code").expect("benchmark regexp should be valid")),
    );
    group.bench_with_input(
      BenchmarkId::new("regexp_with_exclude", hit_percent),
      &hit_percent,
      |bencher, _| {
        bencher.iter(|| {
          for content in &modules {
            black_box(regexp_with_exclude_filter.matches(Some(black_box(content))));
          }
        });
      },
    );
  }

  group.finish();
}
