//! Per-event cost of the ignored-event filter, which the recursive-root watch
//! runs once for every path under the project (mostly deep `node_modules` /
//! build-output). A = ancestor-walk over `fast_glob` (O(depth)); B = one
//! precompiled regex (O(1), the shipped `IgnoredMatcher`). A is kept here only
//! as the baseline B is measured against.

use std::{hint::black_box, path::Path};

use criterion::{Criterion, criterion_group, criterion_main};
use rspack_watcher::{FsWatcherIgnored, IgnoredMatcher};

/// Realistic rstest-style ignored patterns (directory-shaped globs, no `/**`).
const PATTERNS: &[&str] = &[
  "**/node_modules",
  "**/.git",
  "**/dist/.rstest-temp",
  "**/*.snap",
];

fn patterns() -> Vec<String> {
  PATTERNS.iter().map(|s| (*s).to_owned()).collect()
}

/// Baseline A: test the path and every ancestor against the globs via
/// `fast_glob` — the ancestor-walk alternative to the shipped regex matcher.
fn a_is_ignored(ignored: &FsWatcherIgnored, path: &Path) -> bool {
  std::iter::successors(Some(path), |p| p.parent())
    .filter_map(Path::to_str)
    .any(|s| ignored.should_be_ignored(s))
}

// Representative event paths seen under a recursive-root watch.
const KEPT_DEEP: &str = "/proj/packages/core/src/components/widgets/forms/inputs/text/index.ts";
const IGNORED_DEEP: &str = "/proj/node_modules/.pnpm/@scope+pkg@1.2.3/node_modules/@scope/pkg/dist/esm/chunk/very/deep/module.js";
const IGNORED_TEMP: &str = "/proj/dist/.rstest-temp/abc/def/ghi/spec-123.test.mjs";
const KEPT_SHALLOW: &str = "/proj/README.md";
const IGNORED_SHALLOW: &str = "/proj/.git/config";

fn bench_per_event(c: &mut Criterion) {
  let ignored = FsWatcherIgnored::Paths(patterns());
  let matcher = IgnoredMatcher::new(FsWatcherIgnored::Paths(patterns()));

  for (name, path) in [
    ("kept_deep", KEPT_DEEP), // worst case for A: walk ALL ancestors, no match
    ("ignored_deep", IGNORED_DEEP), // deep node_modules file → dropped
    ("ignored_temp", IGNORED_TEMP), // deep build-output file → dropped
    ("kept_shallow", KEPT_SHALLOW), // shallow source file → kept
    ("ignored_shallow", IGNORED_SHALLOW), // .git near root → dropped early
  ] {
    let p = Path::new(path);
    let mut group = c.benchmark_group(name);
    group.bench_function("A_ancestor_walk", |b| {
      b.iter(|| black_box(a_is_ignored(&ignored, black_box(p))));
    });
    group.bench_function("B_precompiled_regex", |b| {
      b.iter(|| black_box(matcher.is_ignored(black_box(path))));
    });
    group.finish();
  }
}

/// Aggregate throughput over a realistic burst: a recursive-root rebuild emits
/// far more `node_modules` / output events than real source edits.
fn bench_event_burst(c: &mut Criterion) {
  let ignored = FsWatcherIgnored::Paths(patterns());
  let matcher = IgnoredMatcher::new(FsWatcherIgnored::Paths(patterns()));

  let mut stream: Vec<&str> = Vec::new();
  stream.extend(std::iter::repeat_n(IGNORED_DEEP, 200));
  stream.extend(std::iter::repeat_n(KEPT_DEEP, 200));
  stream.extend(std::iter::repeat_n(IGNORED_TEMP, 50));
  stream.extend(std::iter::repeat_n(KEPT_SHALLOW, 50));
  let paths: Vec<&Path> = stream.iter().map(|s| Path::new(*s)).collect();

  let mut group = c.benchmark_group("event_burst_500");
  group.bench_function("A_ancestor_walk", |b| {
    b.iter(|| {
      let mut dropped = 0usize;
      for p in &paths {
        if a_is_ignored(&ignored, black_box(p)) {
          dropped += 1;
        }
      }
      black_box(dropped)
    });
  });
  group.bench_function("B_precompiled_regex", |b| {
    b.iter(|| {
      let mut dropped = 0usize;
      for s in &stream {
        if matcher.is_ignored(black_box(s)) {
          dropped += 1;
        }
      }
      black_box(dropped)
    });
  });
  group.finish();
}

/// One-time construction cost: B compiles the regex, A just stores strings.
/// Paid once per `NativeWatcher` and reused across the rebuild loop.
fn bench_construction(c: &mut Criterion) {
  let mut group = c.benchmark_group("construction");
  group.bench_function("A_build_FsWatcherIgnored", |b| {
    b.iter(|| black_box(FsWatcherIgnored::Paths(black_box(patterns()))));
  });
  group.bench_function("B_compile_IgnoredMatcher", |b| {
    b.iter(|| {
      black_box(IgnoredMatcher::new(FsWatcherIgnored::Paths(black_box(
        patterns(),
      ))))
    });
  });
  group.finish();
}

criterion_group!(
  benches,
  bench_per_event,
  bench_event_burst,
  bench_construction
);
criterion_main!(benches);
