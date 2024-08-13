// This benchmark aims to demonstrate that rspack_glob has precompilation.
// When a glob pattern is matched only once, its performance is much lower than fast_glob.
// However, when a glob pattern is matched more than 5 times, rspack_glob's performance surpasses fast_glob.
//
// Matching multiple times aligns with the actual usage scenario in Rspack.

use codspeed_criterion_compat::{criterion_group, criterion_main, Criterion};

const PATH: &str = "some/a/bigger/path/to/the/crazy/needle.txt";
const GLOB: &str = "some/**/needle.txt";

#[inline]
fn rspack_glob(pat: &str, s: &str) -> bool {
  #[allow(clippy::unwrap_used)]
  let pat = rspack_glob::Pattern::new(pat).unwrap();
  pat.matches(s)
}

fn fast_glob_crate(b: &mut Criterion) {
  b.bench_function("fast_glob", |b| {
    b.iter(|| {
      for _ in 0..6 {
        fast_glob::glob_match(GLOB, PATH);
      }
    })
  });
}

fn rspack_glob_crate(b: &mut Criterion) {
  b.bench_function("rspack_glob", |b| b.iter(|| rspack_glob(GLOB, PATH)));
}

criterion_group!(benches, fast_glob_crate, rspack_glob_crate);
criterion_main!(benches);
