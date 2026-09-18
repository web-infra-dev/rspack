use std::alloc::{GlobalAlloc, Layout};

/// Mimalloc with a direct allocation entry point for ordinary alignments.
pub struct MiMalloc;

#[inline]
fn use_unaligned_api(layout: Layout) -> bool {
  // Mimalloc's ordinary blocks are word aligned. Keep requests whose alignment
  // exceeds their size on the explicit aligned API as well.
  layout.align() <= align_of::<usize>() && layout.align() <= layout.size()
}

// SAFETY: Both paths allocate from the same mimalloc instance. The ordinary
// entry points satisfy the alignment checked above; other layouts retain the
// upstream allocator's alignment handling, deallocation, and reallocation.
unsafe impl GlobalAlloc for MiMalloc {
  #[inline]
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    if use_unaligned_api(layout) {
      // SAFETY: The caller supplies a valid nonzero layout. The predicate
      // ensures that mimalloc's ordinary alignment satisfies this layout.
      unsafe { libmimalloc_sys::mi_malloc(layout.size()).cast() }
    } else {
      // SAFETY: Forward the caller's GlobalAlloc contract unchanged.
      unsafe { ::mimalloc::MiMalloc.alloc(layout) }
    }
  }

  #[inline]
  unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
    if use_unaligned_api(layout) {
      // SAFETY: The predicate ensures sufficient alignment; mi_zalloc zeros
      // the requested bytes and preserves null on allocation failure.
      unsafe { libmimalloc_sys::mi_zalloc(layout.size()).cast() }
    } else {
      // SAFETY: Forward the caller's GlobalAlloc contract unchanged.
      unsafe { ::mimalloc::MiMalloc.alloc_zeroed(layout) }
    }
  }

  #[inline]
  unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
    // SAFETY: Both allocation paths return mimalloc-owned pointers.
    unsafe { ::mimalloc::MiMalloc.dealloc(ptr, layout) }
  }

  #[inline]
  unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
    // SAFETY: Both allocation paths return mimalloc-owned pointers; the
    // upstream aligned realloc preserves the requested alignment on moves.
    unsafe { ::mimalloc::MiMalloc.realloc(ptr, layout, new_size) }
  }
}
