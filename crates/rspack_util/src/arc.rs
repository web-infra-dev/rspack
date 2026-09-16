//! Temporary support for converting a standard-library [`Arc`] into [`UniqueArc`].
//!
//! Replace this module with `Arc::try_into_unique` when
//! <https://github.com/rust-lang/rust/pull/150631> reaches our toolchain.
//! This implementation depends on private layouts from rustc commit
//! `e8e4541ff19649d95afab52fdde2c2eaa6829965` (nightly-2026-04-16), checked in
//! `build.rs`. It only supports sized values and the default global allocator.
//! A custom standard library must preserve those layouts as well.

use std::{
  mem::{ManuallyDrop, align_of, offset_of, size_of, transmute},
  ptr::NonNull,
  sync::{
    Arc, UniqueArc,
    atomic::{AtomicUsize, Ordering},
  },
};

// Mirror of alloc::sync::ArcInner in the pinned standard library. The weak
// counter includes the implicit weak reference held by Arc/UniqueArc.
// Never create a reference to this entire struct: only the counters may be
// shared while UniqueArc mutably borrows the data.
#[repr(C, align(2))]
struct ArcInner<T> {
  strong: AtomicUsize,
  weak: AtomicUsize,
  data: T,
}

/// Converts the last strong reference into a uniquely owned pointer in place.
///
/// Returns the original `Arc` unchanged if another strong reference exists.
/// Weak references are preserved, but their `upgrade` calls return `None` while
/// the returned `UniqueArc` exists. After [`UniqueArc::into_arc`], the same weak
/// references can be upgraded again and observe the updated value.
///
/// This does not wait for other readers and does not clone or move the value.
/// A racing weak upgrade either prevents this conversion or observes the
/// exclusive state; it cannot acquire a reader during exclusive access.
///
/// # Safety
///
/// All weak pointers to this allocation must have exactly the same inner type
/// as `T`, including lifetimes. In particular, the `Arc` must not have undergone
/// a subtyping coercion since those weak pointers were created. Otherwise a
/// write through `UniqueArc<T>` could invalidate their original type. Even a
/// `T: 'static` bound would not rule out function-pointer subtyping.
///
/// ```
/// #![feature(unique_rc_arc)]
/// let arc = std::sync::Arc::new(1);
/// let weak = std::sync::Arc::downgrade(&arc);
/// // SAFETY: arc and weak both have the same inner type, i32.
/// let mut unique = unsafe { rspack_util::arc::try_into_unique(arc) }.expect("readers finished");
/// assert!(weak.upgrade().is_none());
/// *unique = 2;
/// let arc = std::sync::UniqueArc::into_arc(unique);
/// assert_eq!(*weak.upgrade().expect("shared again"), 2);
/// ```
#[inline]
pub unsafe fn try_into_unique<T>(arc: Arc<T>) -> Result<UniqueArc<T>, Arc<T>> {
  const {
    assert!(size_of::<UniqueArc<T>>() == size_of::<NonNull<ArcInner<T>>>());
    assert!(align_of::<UniqueArc<T>>() == align_of::<NonNull<ArcInner<T>>>());
  }

  let arc = ManuallyDrop::new(arc);
  let data = Arc::as_ptr(&arc);

  // SAFETY: In the pinned std, Arc::as_ptr retains the allocation's raw/mutable
  // provenance. ArcInner is repr(C), so offset_of includes any padding needed
  // for T (including over-aligned and zero-sized T). The subtraction stays in
  // the live Arc allocation and recovers its non-null, aligned header pointer.
  let inner = unsafe {
    data
      .byte_sub(offset_of!(ArcInner<T>, data))
      .cast::<ArcInner<T>>()
      .cast_mut()
  };

  // SAFETY: Only borrow the atomic counter, not the data. This Arc keeps the
  // allocation alive even while other strong and weak references are dropped.
  let strong = unsafe { &(*inner).strong };
  if strong
    .compare_exchange(1, 0, Ordering::Acquire, Ordering::Relaxed)
    .is_err()
  {
    return Err(ManuallyDrop::into_inner(arc));
  }

  // SAFETY: The successful CAS excludes every other strong reference and
  // prevents Weak::upgrade from creating one. Acquire synchronizes with prior
  // Arc drops. We retain the implicit weak reference and the initialized data;
  // ManuallyDrop prevents Arc from decrementing the zero strong count.
  //
  // In the pinned std, UniqueArc<T, Global> consists of exactly one NonNull
  // allocation pointer plus zero-sized PhantomData/Global fields. Transfer that
  // pointer into UniqueArc, whose Drop destroys the data and releases the
  // implicit weak reference, or whose into_arc publishes writes with Release.
  // The size/alignment checks above supplement the source audit in build.rs;
  // they alone would not justify this private-layout conversion.
  // The caller ensures that mutation cannot invalidate a weak pointer's type.
  unsafe {
    Ok(transmute::<NonNull<ArcInner<T>>, UniqueArc<T>>(
      NonNull::new_unchecked(inner),
    ))
  }
}
