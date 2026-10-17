use std::sync::Arc;

use rspack_sources::{ConcatSource, RawStringSource, Source};

#[test]
fn clones_share_optimized_children() {
  let source = ConcatSource::new([
    RawStringSource::from("left"),
    RawStringSource::from("right"),
  ]);
  let clone = source.clone();

  let original_child = source.children()[0].clone();
  let cloned_child = clone.children()[0].clone();
  assert!(Arc::ptr_eq(&original_child, &cloned_child));
}

#[test]
fn large_raw_groups_keep_their_shared_children() {
  let left = format!("{}{}", "a".repeat(40 * 1024), "left");
  let right = format!("{}{}", "b".repeat(40 * 1024), "right");
  let source = ConcatSource::new([
    RawStringSource::from(left.clone()),
    RawStringSource::from(right.clone()),
  ]);

  assert_eq!(source.children().len(), 2);
  assert_eq!(
    source.source().into_string_lossy(),
    format!("{left}{right}")
  );
}

#[test]
fn clones_remain_independent_after_optimization() {
  let source = ConcatSource::new([
    RawStringSource::from("left"),
    RawStringSource::from("right"),
  ]);
  let clone = source.clone();
  assert_eq!(source.source().into_string_lossy(), "leftright");

  let mut updated = clone;
  updated.add(RawStringSource::from("!"));

  assert_eq!(source.source().into_string_lossy(), "leftright");
  assert_eq!(updated.source().into_string_lossy(), "leftright!");
}

#[test]
fn clones_remain_independent_before_optimization() {
  let source = ConcatSource::new([
    RawStringSource::from("left"),
    RawStringSource::from("right"),
  ]);
  let mut clone = source.clone();
  clone.add(RawStringSource::from("!"));

  assert_eq!(source.source().into_string_lossy(), "leftright");
  assert_eq!(clone.source().into_string_lossy(), "leftright!");
}
