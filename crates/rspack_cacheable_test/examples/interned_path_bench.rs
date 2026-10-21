//! Micro-benchmark mirroring PR #15910: 8,192 references to 96-byte
//! paths at 1, 128 and 8,192 distinct paths, with and without a portable root.
use std::{
  hint::black_box,
  path::{Path, PathBuf},
  time::Instant,
};

use rspack_cacheable::{CacheableContext, enable_cacheable as cacheable, from_bytes, to_bytes};
use rspack_paths::InternedPath;

#[cacheable]
struct Deps {
  paths: Vec<InternedPath>,
}

struct Ctx(Option<PathBuf>);

impl CacheableContext for Ctx {
  fn project_root(&self) -> Option<&Path> {
    self.0.as_deref()
  }
}

const REFS: usize = 8192;
const ROOT: &str = "/home/user/project";
const BATCHES: usize = 21;
const OPS: usize = 50;

fn make_path(i: usize) -> InternedPath {
  let mut s = format!("{ROOT}/src/components/feature/{i:06}/");
  while s.len() < 93 {
    s.push('x');
  }
  s.push_str(".js");
  assert_eq!(s.len(), 96);
  InternedPath::from(s.as_str())
}

/// (median, min) over batches. Under machine load the min is the better estimate.
fn summarize(mut v: Vec<f64>) -> (f64, f64) {
  v.sort_by(|a, b| {
    a.partial_cmp(b)
      .expect("benchmark durations should not be NaN")
  });
  (v[v.len() / 2], v[0])
}

fn time_batches(mut f: impl FnMut()) -> (f64, f64) {
  let mut per_op = Vec::with_capacity(BATCHES);
  for _ in 0..BATCHES {
    let start = Instant::now();
    for _ in 0..OPS {
      f();
    }
    per_op.push(start.elapsed().as_secs_f64() * 1e6 / OPS as f64);
  }
  summarize(per_op)
}

fn main() {
  println!(
    "| Root | Distinct | Serialize µs (median / min) | Warm deserialize µs (median / min) | Archive (B) |"
  );
  println!("| --- | ---: | ---: | ---: | ---: |");
  for root in [false, true] {
    let ctx = Ctx(root.then(|| PathBuf::from(ROOT)));
    for distinct in [1usize, 128, 8192] {
      let paths: Vec<InternedPath> = (0..REFS).map(|i| make_path(i % distinct)).collect();
      let deps = Deps { paths };
      let bytes = to_bytes(&deps, &ctx).expect("should serialize benchmark paths");
      let restored: Deps = from_bytes(&bytes, &ctx).expect("should deserialize benchmark paths");
      assert_eq!(restored.paths, deps.paths);
      let ser = time_batches(|| {
        black_box(to_bytes(black_box(&deps), &ctx).expect("should serialize benchmark paths"));
      });
      let de = time_batches(|| {
        black_box(
          from_bytes::<Deps, _>(black_box(&bytes), &ctx)
            .expect("should deserialize benchmark paths"),
        );
      });
      println!(
        "| {} | {distinct} | {:.1} / {:.1} | {:.1} / {:.1} | {} |",
        if root { "set" } else { "unset" },
        ser.0,
        ser.1,
        de.0,
        de.1,
        bytes.len()
      );
    }
  }
}
