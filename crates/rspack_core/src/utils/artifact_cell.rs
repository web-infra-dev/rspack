use std::{
  marker::PhantomData,
  ops::Deref,
  ptr::NonNull,
  sync::atomic::{AtomicIsize, Ordering},
};

const UNUSED: isize = 0;
const WRITING: isize = -1;

/// A cell for artifacts that allows controlled interior mutability.
///
/// This type stores the artifact on the heap and provides:
/// - Safe immutable access via `get()`
/// - Controlled mutable access via `with_mut()` which uses a closure to scope mutations
/// - Runtime borrow checking to prevent simultaneous mutable and immutable access
///
/// # Runtime Checking
///
/// Unlike raw pointers, `ArtifactCell` performs runtime checks to prevent:
/// - Accessing via `get()` while `with_mut()` is executing
/// - Nested `with_mut()` calls
///
/// These checks will panic if violated, similar to `RefCell`.
///
/// # Usage
///
/// This is designed to be used in Compilation to store artifacts where:
/// 1. The artifact needs to be mutated while having read-only access to compilation
/// 2. JS binding layer needs to access both artifact and compilation simultaneously
///
/// The `with_mut()` method ensures mutations are scoped to a closure, preventing
/// references from escaping and ensuring memory safety.
///
/// # Safety
///
/// The internal `get_mut_unchecked()` method (hidden from docs) bypasses runtime
/// checks and is available for binding layer use where NAPI's single-threaded
/// model ensures safety.
///
/// # Examples
///
/// ```ignore
/// use rspack_core::ArtifactCell;
///
/// pub struct Compilation {
///     pub my_artifact: ArtifactCell<MyArtifact>,
///     pub chunk_graph: ChunkGraph,
/// }
///
/// impl Compilation {
///     // Recommended: Provide a method that accepts a closure with compilation access
///     pub fn with_artifact_mut<F, R>(&self, f: F) -> R
///     where
///         F: FnOnce(&mut MyArtifact, &Compilation) -> R,
///     {
///         self.my_artifact.with_mut(|artifact| f(artifact, self))
///     }
///
///     pub fn update_artifact(&self) {
///         self.with_artifact_mut(|artifact, compilation| {
///             // This will panic if artifact tries to call get() on itself
///             // let _self_ref = compilation.my_artifact.get(); // ❌ Panic!
///
///             artifact.update(&compilation.chunk_graph);
///         });
///     }
///
///     // Or use with_mut directly
///     pub fn simple_update(&self) {
///         self.my_artifact.with_mut(|artifact| {
///             artifact.clear();
///         });
///     }
/// }
/// ```
#[derive(Debug)]
pub struct ArtifactCell<T> {
  ptr: NonNull<T>,
  borrow: AtomicIsize,
  _marker: PhantomData<Box<T>>,
}

impl<T> ArtifactCell<T> {
  /// Creates a new `ArtifactCell` by moving the value onto the heap.
  #[inline]
  pub fn new(value: T) -> Self {
    let boxed = Box::new(value);
    let ptr = NonNull::new(Box::into_raw(boxed)).expect("Box::into_raw should never return null");
    Self {
      ptr,
      borrow: AtomicIsize::new(UNUSED),
      _marker: PhantomData,
    }
  }

  /// Returns a shared reference to the contained value.
  ///
  /// This is always safe to call and can be used to read the artifact.
  ///
  /// # Panics
  ///
  /// Panics if the value is currently being mutated via `with_mut()`.
  ///
  /// # Examples
  ///
  /// ```ignore
  /// let cell = ArtifactCell::new(42);
  /// let value = cell.get();
  /// assert_eq!(*value, 42);
  ///
  /// // This will panic:
  /// cell.with_mut(|_| {
  ///     let _ = cell.get(); // ❌ Panic: already mutably borrowed
  /// });
  /// ```
  #[inline]
  pub fn get(&self) -> &T {
    // Check if currently being mutated
    let borrow = self.borrow.load(Ordering::Acquire);
    if borrow == WRITING {
      panic!("ArtifactCell: attempted to borrow immutably while already borrowed mutably");
    }

    // SAFETY: The pointer is valid for the lifetime of self,
    // as it's created from Box and only dropped in Drop::drop.
    // We've checked that no mutable borrow exists.
    unsafe { self.ptr.as_ref() }
  }

  /// Provides controlled mutable access to the contained value through a closure.
  ///
  /// This is the recommended way to mutate the artifact, as it ensures
  /// the mutable reference is scoped to the closure execution.
  ///
  /// # Panics
  ///
  /// Panics if:
  /// - The value is already being mutated (nested `with_mut()` calls)
  /// - The value is being accessed via `get()` during mutation
  ///
  /// # Examples
  ///
  /// ```ignore
  /// let cell = ArtifactCell::new(MyArtifact::default());
  /// cell.with_mut(|artifact| {
  ///     artifact.update_something();
  /// });
  ///
  /// // This will panic:
  /// cell.with_mut(|_| {
  ///     cell.with_mut(|_| {}); // ❌ Panic: already mutably borrowed
  /// });
  /// ```
  #[inline]
  pub fn with_mut<F, R>(&self, f: F) -> R
  where
    F: FnOnce(&mut T) -> R,
  {
    // Try to acquire mutable borrow
    let prev = self.borrow.swap(WRITING, Ordering::Acquire);
    if prev != UNUSED {
      // Restore the previous state before panicking
      self.borrow.store(prev, Ordering::Release);
      panic!("ArtifactCell: attempted to borrow mutably while already borrowed");
    }

    // Create a guard to ensure we release the borrow even if f panics
    struct BorrowGuard<'a> {
      borrow: &'a AtomicIsize,
    }

    impl<'a> Drop for BorrowGuard<'a> {
      fn drop(&mut self) {
        self.borrow.store(UNUSED, Ordering::Release);
      }
    }

    let _guard = BorrowGuard {
      borrow: &self.borrow,
    };

    // SAFETY: We have exclusive access (checked via atomic flag)
    let value = unsafe { &mut *self.ptr.as_ptr() };
    f(value)
  }

  /// Returns a mutable reference to the contained value.
  ///
  /// # Safety
  ///
  /// This is an internal method primarily for binding layer use.
  /// The caller must ensure that:
  /// - No other references (mutable or immutable) to the contained value exist
  /// - The returned reference's lifetime doesn't overlap with other references
  ///
  /// For normal use, prefer `with_mut()` which provides scoped access.
  #[doc(hidden)]
  #[inline]
  #[allow(clippy::mut_from_ref)]
  pub fn get_mut_unchecked(&self) -> &mut T {
    // SAFETY: Caller must uphold the safety contract
    unsafe { &mut *self.ptr.as_ptr() }
  }

  /// Returns a raw pointer to the contained value.
  ///
  /// The caller must ensure they do not use the pointer after the cell is dropped.
  ///
  /// # Safety
  ///
  /// This is primarily for advanced use cases in the binding layer.
  #[inline]
  pub fn as_ptr(&self) -> *const T {
    self.ptr.as_ptr()
  }

  /// Returns a mutable raw pointer to the contained value.
  ///
  /// The caller must ensure they do not use the pointer after the cell is dropped,
  /// and must uphold Rust's aliasing rules.
  ///
  /// # Safety
  ///
  /// This is primarily for advanced use cases in the binding layer.
  #[inline]
  pub fn as_mut_ptr(&self) -> *mut T {
    self.ptr.as_ptr()
  }

  /// Consumes the cell and returns the inner value.
  #[allow(clippy::disallowed_methods)]
  pub fn into_inner(self) -> T {
    // SAFETY: We own self, so we can take ownership of the Box
    let boxed = unsafe { Box::from_raw(self.ptr.as_ptr()) };
    let result = *boxed;
    // Prevent double-free
    std::mem::forget(self);
    result
  }

  /// Replaces the contained value with a new value and returns the old value.
  ///
  /// This is atomic and ensures the cell always contains a valid value.
  ///
  /// # Examples
  ///
  /// ```ignore
  /// let cell = ArtifactCell::new(42);
  /// let old_value = cell.replace(100);
  /// assert_eq!(old_value, 42);
  /// assert_eq!(*cell.get(), 100);
  /// ```
  pub fn replace(&self, mut new_value: T) -> T {
    // SAFETY: We're atomically swapping the values, so the cell
    // always contains a valid value. This is similar to Cell::replace.
    unsafe {
      std::ptr::swap(self.ptr.as_ptr(), &mut new_value);
    }
    new_value
  }

  /// Takes the value out of the cell, replacing it with the default value.
  ///
  /// This is equivalent to `replace(T::default())` but more convenient.
  ///
  /// # Examples
  ///
  /// ```ignore
  /// let cell = ArtifactCell::new(vec![1, 2, 3]);
  /// let value = cell.take();
  /// assert_eq!(value, vec![1, 2, 3]);
  /// assert!(cell.get().is_empty());
  /// ```
  pub fn take(&self) -> T
  where
    T: Default,
  {
    self.replace(T::default())
  }
}

impl<T: Default> Default for ArtifactCell<T> {
  fn default() -> Self {
    Self::new(T::default())
  }
}

impl<T> From<T> for ArtifactCell<T> {
  fn from(value: T) -> Self {
    Self::new(value)
  }
}

impl<T> Drop for ArtifactCell<T> {
  fn drop(&mut self) {
    // SAFETY: ptr was created from Box::into_raw and is still valid
    unsafe {
      let _ = Box::from_raw(self.ptr.as_ptr());
    }
  }
}

// SAFETY: ArtifactCell can be Send if T is Send,
// as it owns the T through a Box (which is Send when T is Send)
unsafe impl<T: Send> Send for ArtifactCell<T> {}

// SAFETY: ArtifactCell can be Sync if T is Sync,
// as shared access (&self.get()) is safe when T is Sync
// Note: get_mut requires explicit unsafe awareness at call site
unsafe impl<T: Sync> Sync for ArtifactCell<T> {}

// Implement AsRef trait for ergonomic access
impl<T> AsRef<T> for ArtifactCell<T> {
  fn as_ref(&self) -> &T {
    self.get()
  }
}

// Implement Deref to allow transparent read-only access to the contained value
impl<T> Deref for ArtifactCell<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    self.get()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_new_and_get() {
    let cell = ArtifactCell::new(42);
    assert_eq!(*cell.get(), 42);
  }

  #[test]
  fn test_with_mut() {
    let cell = ArtifactCell::new(42);
    cell.with_mut(|value| {
      *value = 100;
    });
    assert_eq!(*cell.get(), 100);
  }

  #[test]
  fn test_with_mut_return_value() {
    let cell = ArtifactCell::new(vec![1, 2, 3]);
    let len = cell.with_mut(|vec| {
      vec.push(4);
      vec.len()
    });
    assert_eq!(len, 4);
    assert_eq!(cell.get().len(), 4);
  }

  #[test]
  fn test_into_inner() {
    let cell = ArtifactCell::new(String::from("hello"));
    let value = cell.into_inner();
    assert_eq!(value, "hello");
  }

  #[test]
  fn test_default() {
    let cell = ArtifactCell::<i32>::default();
    assert_eq!(*cell.get(), 0);
  }

  #[test]
  fn test_from() {
    let cell = ArtifactCell::from(42);
    assert_eq!(*cell.get(), 42);
  }

  #[test]
  fn test_as_ref_trait() {
    let cell = ArtifactCell::new(String::from("test"));
    let s: &String = cell.as_ref();
    assert_eq!(s, "test");
  }

  #[derive(Debug, Default)]
  struct TestArtifact {
    data: Vec<String>,
    count: usize,
  }

  #[test]
  fn test_complex_type() {
    let artifact = TestArtifact {
      data: vec!["a".to_string(), "b".to_string()],
      count: 2,
    };
    let cell = ArtifactCell::new(artifact);
    assert_eq!(cell.get().data.len(), 2);
    assert_eq!(cell.get().count, 2);
  }

  #[test]
  fn test_mutation_pattern() {
    let cell = ArtifactCell::new(TestArtifact::default());

    // Simulate the with_mut pattern
    cell.with_mut(|artifact| {
      artifact.data.push("item1".to_string());
      artifact.count += 1;
    });

    assert_eq!(cell.get().count, 1);

    cell.with_mut(|artifact| {
      artifact.data.push("item2".to_string());
      artifact.count += 1;
    });

    assert_eq!(cell.get().count, 2);
    assert_eq!(cell.get().data.len(), 2);
  }

  // Test that Send/Sync are properly implemented
  #[test]
  fn test_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<ArtifactCell<String>>();
    assert_sync::<ArtifactCell<String>>();
  }

  #[test]
  fn test_concurrent_reads() {
    let cell = ArtifactCell::new(vec![1, 2, 3]);

    // Multiple immutable references are safe
    let ref1 = cell.get();
    let ref2 = cell.get();
    let ref3 = cell.get();

    assert_eq!(ref1.len(), ref2.len());
    assert_eq!(ref2.len(), ref3.len());
    assert_eq!(ref1.len(), 3);
  }

  #[test]
  #[should_panic(expected = "attempted to borrow immutably while already borrowed mutably")]
  fn test_get_during_with_mut_panics() {
    let cell = ArtifactCell::new(42);
    cell.with_mut(|_value| {
      // This should panic
      let _read = cell.get();
    });
  }

  #[test]
  #[should_panic(expected = "attempted to borrow mutably while already borrowed")]
  fn test_nested_with_mut_panics() {
    let cell = ArtifactCell::new(42);
    cell.with_mut(|_outer| {
      // This should panic
      cell.with_mut(|_inner| {});
    });
  }

  #[test]
  fn test_with_mut_panic_recovery() {
    let cell = ArtifactCell::new(vec![1, 2, 3]);

    // First with_mut that panics
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
      cell.with_mut(|_| {
        panic!("intentional panic");
      });
    }));
    assert!(result.is_err());

    // Should be able to use the cell again after panic
    cell.with_mut(|vec| {
      vec.push(4);
    });

    assert_eq!(cell.get().len(), 4);
  }

  #[test]
  fn test_sequential_with_mut_ok() {
    let cell = ArtifactCell::new(0);

    // Sequential with_mut calls are fine
    cell.with_mut(|value| {
      *value = 1;
    });

    cell.with_mut(|value| {
      *value = 2;
    });

    assert_eq!(*cell.get(), 2);
  }

  #[test]
  fn test_replace() {
    let cell = ArtifactCell::new(42);
    let old_value = cell.replace(100);
    assert_eq!(old_value, 42);
    assert_eq!(*cell.get(), 100);
  }

  #[test]
  fn test_take() {
    let cell = ArtifactCell::new(vec![1, 2, 3]);
    let value = cell.take();
    assert_eq!(value, vec![1, 2, 3]);
    assert!(cell.get().is_empty());
  }

  #[test]
  fn test_replace_maintains_validity() {
    let cell = ArtifactCell::new(String::from("first"));

    // Multiple replacements
    let first = cell.replace(String::from("second"));
    assert_eq!(first, "first");
    assert_eq!(cell.get(), "second");

    let second = cell.replace(String::from("third"));
    assert_eq!(second, "second");
    assert_eq!(cell.get(), "third");
  }

  #[test]
  fn test_deref() {
    let cell = ArtifactCell::new(vec![1, 2, 3]);

    // Can use methods directly via Deref
    assert_eq!(cell.len(), 3);
    assert_eq!(cell[0], 1);
    assert!(!cell.is_empty());
  }

  #[test]
  fn test_deref_with_struct() {
    let cell = ArtifactCell::new(TestArtifact {
      data: vec!["a".to_string(), "b".to_string()],
      count: 2,
    });

    // Can access fields directly via Deref
    assert_eq!(cell.data.len(), 2);
    assert_eq!(cell.count, 2);
  }
}
