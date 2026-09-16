#![feature(unique_rc_arc)]

use std::{
  sync::{
    Arc, Barrier, UniqueArc,
    atomic::{AtomicUsize, Ordering},
  },
  thread,
};

use rspack_util::arc::try_into_unique;

#[test]
fn preserves_allocation_and_weak_references_across_round_trips() {
  let mut arc = Arc::new(String::from("initial"));
  let weak = Arc::downgrade(&arc);
  let pointer = Arc::as_ptr(&arc);

  for _ in 0..4 {
    // SAFETY: every pointer has the same inner type, String.
    let mut unique = unsafe { try_into_unique(arc) }.expect("only one strong reference");
    let another_weak = UniqueArc::downgrade(&unique);
    assert!(weak.upgrade().is_none());
    assert!(another_weak.upgrade().is_none());
    unique.push('!');
    arc = UniqueArc::into_arc(unique);
    assert_eq!(Arc::as_ptr(&arc), pointer);
    assert!(Arc::ptr_eq(&arc, &weak.upgrade().expect("shared again")));
    assert!(Arc::ptr_eq(
      &arc,
      &another_weak.upgrade().expect("shared again")
    ));
    assert_eq!(Arc::strong_count(&arc), 1);
  }
  assert_eq!(&*arc, "initial!!!!");
}

#[test]
fn failure_returns_the_original_reference_without_decrementing() {
  let arc = Arc::new(42);
  let reader = Arc::clone(&arc);
  let weak = Arc::downgrade(&arc);
  // SAFETY: every pointer has the same inner type, i32.
  let arc = unsafe { try_into_unique(arc) }.expect_err("reader is still alive");
  assert!(Arc::ptr_eq(&arc, &reader));
  assert_eq!(Arc::strong_count(&arc), 2);
  drop(reader);
  // SAFETY: every pointer still has the same inner type, i32.
  let unique = unsafe { try_into_unique(arc) }.expect("reader has finished");
  assert!(weak.upgrade().is_none());
  assert_eq!(*unique, 42);
}

#[derive(Debug)]
struct DropProbe(Arc<AtomicUsize>);

impl Drop for DropProbe {
  fn drop(&mut self) {
    self.0.fetch_add(1, Ordering::Relaxed);
  }
}

#[test]
fn drops_value_once_with_and_without_outstanding_weak_references() {
  for keep_weak in [false, true] {
    let drops = Arc::new(AtomicUsize::new(0));
    let arc = Arc::new(DropProbe(Arc::clone(&drops)));
    let weak = keep_weak.then(|| Arc::downgrade(&arc));
    // SAFETY: every pointer has the same inner type, DropProbe.
    let unique = unsafe { try_into_unique(arc) }.expect("only one strong reference");
    assert_eq!(drops.load(Ordering::Relaxed), 0);
    drop(unique);
    assert_eq!(drops.load(Ordering::Relaxed), 1);
    if let Some(weak) = weak {
      assert!(weak.upgrade().is_none());
      drop(weak);
    }
    assert_eq!(drops.load(Ordering::Relaxed), 1);
  }
}

#[test]
fn supports_zero_sized_and_over_aligned_values() {
  #[derive(Debug)]
  #[repr(align(4096))]
  struct Aligned(u8);

  #[derive(Debug)]
  #[repr(align(4096))]
  struct AlignedZst;

  // SAFETY: none of these allocations has weak pointers.
  let mut unique = unsafe { try_into_unique(Arc::new(Aligned(1))) }.expect("unique");
  unique.0 = 2;
  assert_eq!(UniqueArc::into_arc(unique).0, 2);
  drop(unsafe { try_into_unique(Arc::new(AlignedZst)) }.expect("unique"));
  drop(unsafe { try_into_unique(Arc::new(())) }.expect("unique"));
}

#[test]
fn weak_upgrade_racing_conversion_cannot_read_during_mutation() {
  let iterations = if cfg!(miri) { 8 } else { 256 };
  for _ in 0..iterations {
    let arc = Arc::new(0);
    let weak = Arc::downgrade(&arc);
    let barrier = Arc::new(Barrier::new(2));
    thread::scope(|scope| {
      let barrier = &barrier;
      let reader = scope.spawn(move || {
        barrier.wait();
        let reader = weak.upgrade();
        barrier.wait();
        reader
      });
      barrier.wait();
      // SAFETY: every pointer has the same inner type, i32.
      let result = unsafe { try_into_unique(arc) };
      // Both results remain alive until both operations have completed.
      barrier.wait();
      let reader = reader.join().expect("reader should not panic");
      match result {
        Ok(mut unique) => {
          assert!(reader.is_none());
          *unique = 1;
          assert_eq!(*UniqueArc::into_arc(unique), 1);
        }
        Err(arc) => {
          let reader = reader.expect("upgrade won the race");
          assert!(Arc::ptr_eq(&arc, &reader));
          assert_eq!(*reader, 0);
        }
      }
    });
  }
}
