#![cfg(allocative)]

use std::{
  collections::HashMap,
  mem::size_of,
  sync::{
    Arc, LazyLock, Mutex,
    atomic::{AtomicBool, Ordering},
  },
};

use allocative::{Allocative, FlameGraphBuilder};
use rspack_sources::{BoxSource, CachedSource, RawStringSource, SourceExt};
use smol_str::SmolStr;

fn capture(value: &dyn Allocative) -> (usize, String, String) {
  let mut builder = FlameGraphBuilder::with_shared_ownership();
  builder.visit_root(value);
  let output = builder.finish();
  let folded = output.flamegraph().write();
  let total = folded
    .lines()
    .map(|line| {
      line
        .rsplit_once(' ')
        .expect("folded row has a weight")
        .1
        .parse::<usize>()
        .expect("weight is an integer")
    })
    .sum();
  (total, folded, output.warnings())
}

#[test]
fn reserved_vector_capacity_is_counted_without_elements() {
  let value: Vec<u64> = Vec::with_capacity(128);
  let (total, _, warnings) = capture(&value);
  assert_eq!(
    total,
    size_of::<Vec<u64>>() + value.capacity() * size_of::<u64>()
  );
  assert!(warnings.is_empty(), "{warnings}");
}

#[test]
fn hash_map_accounts_for_capacity_and_nested_allocations() {
  let mut value = HashMap::with_capacity(64);
  let text = String::with_capacity(4096);
  let text_capacity = text.capacity();
  value.insert(1u64, text);
  let (total, _, warnings) = capture(&value);
  // Standard HashMap exposes usable capacity, not its private control-byte layout.
  assert_eq!(
    total,
    size_of_val(&value) + value.capacity() * size_of::<(u64, String)>() + text_capacity
  );
  assert!(warnings.is_empty(), "{warnings}");
}

// Deliberately test both the Arc allocation and its separately owned String buffer.
#[allow(clippy::rc_buffer)]
#[derive(Allocative)]
struct Owners {
  first: Arc<String>,
  second: Arc<String>,
}

#[test]
fn shared_allocations_stay_below_the_first_owner_and_are_counted_once() {
  let text = Arc::new("x".repeat(4096));
  let single = capture(&text).0;
  let value = Owners {
    first: text.clone(),
    second: text,
  };
  let (total, folded, warnings) = capture(&value);
  assert_eq!(total, single + size_of::<Arc<String>>());
  assert!(
    folded
      .lines()
      .any(|line| line.contains("Owners;first;") && line.contains("capacity")),
    "{folded}"
  );
  assert!(
    !folded
      .lines()
      .any(|line| line.contains(";second;") && line.contains("capacity")),
    "{folded}"
  );
  assert!(warnings.is_empty(), "{warnings}");
}

#[derive(Allocative)]
struct Node {
  next: Mutex<Option<Arc<Node>>>,
  bytes: Vec<u8>,
}

#[test]
fn strong_cycles_terminate_and_deduplicate() {
  let first = Arc::new(Node {
    next: Mutex::new(None),
    bytes: vec![0; 4096],
  });
  let second = Arc::new(Node {
    next: Mutex::new(Some(first.clone())),
    bytes: vec![0; 8192],
  });
  *first.next.lock().unwrap() = Some(second.clone());
  let (total, _, warnings) = capture(&first);
  *first.next.lock().unwrap() = None;
  *second.next.lock().unwrap() = None;
  assert!(total >= 4096 + 8192);
  assert!(total < 4096 + 8192 + 1024);
  assert!(warnings.is_empty(), "{warnings}");
}

#[test]
fn shared_smol_strings_are_not_counted_twice() {
  let value = SmolStr::new("x".repeat(4096));
  assert!(value.is_heap_allocated());
  let single = capture(&value).0;
  let (total, _, warnings) = capture(&(value.clone(), value));
  assert_eq!(total, single + size_of::<SmolStr>());
  assert!(warnings.is_empty(), "{warnings}");
}

#[test]
fn profiling_does_not_initialize_lazy_values() {
  static INITIALIZED: AtomicBool = AtomicBool::new(false);
  let value: LazyLock<Vec<u8>> = LazyLock::new(|| {
    INITIALIZED.store(true, Ordering::Relaxed);
    vec![0; 4096]
  });
  let (total, _, warnings) = capture(&value);
  assert!(!INITIALIZED.load(Ordering::Relaxed));
  assert_eq!(total, size_of_val(&value));
  assert!(warnings.is_empty(), "{warnings}");
}

#[test]
fn cached_sources_follow_the_owned_source_without_rendering_it() {
  let raw: BoxSource = RawStringSource::from("x".repeat(4096)).boxed();
  let cache = CachedSource::new(raw.clone());
  let single = capture(&raw).0;
  let (total, _, warnings) = capture(&(raw, cache));
  assert!(total >= single);
  assert!(
    total < single + 2048,
    "source text was copied or counted twice: {total}"
  );
  assert!(warnings.is_empty(), "{warnings}");
}

#[test]
fn locked_values_report_missing_coverage_without_blocking() {
  let value = Mutex::new(vec![0u8; 4096]);
  let _guard = value.lock().expect("test lock");
  let (total, _, warnings) = capture(&value);
  assert_eq!(total, size_of_val(&value));
  assert!(
    warnings.contains("Unobserved nested allocations"),
    "{warnings}"
  );
}
