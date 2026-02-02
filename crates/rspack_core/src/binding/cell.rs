// This crate introduces a smart pointer called BindingCell.
//
// When the "napi" feature is disabled, BindingCell uses the sys_binding module as a simple alias for Box.
//
// When the "napi" feature is enabled, BindingCell uses the napi_binding module.
// 1. BindingCell establishes a 1:1 relationship between a Rust instance and a JS Object.
// 2. BindingCell creates a JS Object only when converting a Rust instance to a JS object.
// 3. The JS Object is hold a weak reference to the Rust instance. If the Rust instance is dropped,
//    any attempt to access the JS Object will throw an exception indicating that the Rust instance has been dropped.

#[cfg(feature = "napi")]
mod napi_binding {
  use std::{
    alloc::Layout,
    any::{Any, TypeId},
    hash::{Hash, Hasher},
    hint,
    marker::CoercePointee,
    ops::{Deref, DerefMut},
    ptr::{self, NonNull},
    sync::atomic::{
      AtomicUsize,
      Ordering::{Acquire, Relaxed, Release},
    },
  };

  use derive_more::Debug;
  use napi::{
    Env,
    bindgen_prelude::{Object, ToNapiValue},
  };
  use once_cell::sync::OnceCell;
  use rspack_napi::{ThreadsafeOneShotRef, object_assign};
  use rspack_sources::BoxSource;
  use rustc_hash::FxHashMap;

  use crate::{
    AssetInfo, CodeGenerationResult, CodeGenerationResults, CompilationAsset, SourceType,
    with_thread_local_allocator,
  };

  /// A soft limit on the amount of references that may be made to an `BindingCell`.
  ///
  /// Going above this limit will abort your program (although not
  /// necessarily) at _exactly_ `MAX_REFCOUNT + 1` references.
  /// Trying to go above it might call a `panic` (if not actually going above it).
  ///
  /// This is a global invariant, and also applies when using a compare-exchange loop.
  const MAX_REFCOUNT: usize = (isize::MAX) as usize;

  /// The error in case either counter reaches above `MAX_REFCOUNT`, and we can `panic` safely.
  const INTERNAL_OVERFLOW_ERROR: &str = "BindingCell counter overflow";

  fn is_dangling<T: ?Sized>(ptr: *const T) -> bool {
    (ptr.cast::<()>()).addr() == usize::MAX
  }

  /// Helper type to allow accessing the reference counts without
  /// making any assertions about the data field.
  struct WeakInner<'a> {
    weak: &'a AtomicUsize,
    strong: &'a AtomicUsize,
  }

  pub struct Reflector<T: ?Sized + 'static> {
    ptr: NonNull<BindingCellInner<T>>,
  }

  impl<T: ?Sized> Reflector<T> {
    pub fn upgrade(&self) -> Option<BindingCell<T>> {
      #[inline]
      fn checked_increment(n: usize) -> Option<usize> {
        // Any write of 0 we can observe leaves the field in permanently zero state.
        if n == 0 {
          return None;
        }
        // See comments in `Arc::clone` for why we do this (for `mem::forget`).
        assert!(n <= MAX_REFCOUNT, "{}", INTERNAL_OVERFLOW_ERROR);
        Some(n + 1)
      }

      // We use a CAS loop to increment the strong count instead of a
      // fetch_add as this function should never take the reference count
      // from zero to one.
      //
      // Relaxed is fine for the failure case because we don't have any expectations about the new state.
      // Acquire is necessary for the success case to synchronise with `BindingCell::new_cyclic`, when the inner
      // value can be initialized after `Weak` references have already been created. In that case, we
      // expect to observe the fully initialized value.
      if self
        .inner()?
        .strong
        .fetch_update(Acquire, Relaxed, checked_increment)
        .is_ok()
      {
        // SAFETY: pointer is not null, verified in checked_increment
        unsafe { Some(BindingCell::from_inner(self.ptr)) }
      } else {
        None
      }
    }

    /// Returns `None` when the pointer is dangling and there is no allocated `BindingCellInner`,
    /// (i.e., when this `Weak` was created by `Weak::new`).
    #[inline]
    fn inner(&self) -> Option<WeakInner<'_>> {
      let ptr = self.ptr.as_ptr();
      if is_dangling(ptr) {
        None
      } else {
        // We are careful to *not* create a reference covering the "data" field, as
        // the field may be mutated concurrently (for example, if the last `BindingCell`
        // is dropped, the data field will be dropped in-place).
        Some(unsafe {
          WeakInner {
            strong: &(*ptr).strong,
            weak: &(*ptr).weak,
          }
        })
      }
    }
  }

  impl<T: ?Sized> Reflector<T> {
    pub fn set_jsobject(&self, env: &Env, object: Object) -> napi::Result<()> {
      let binding = self.upgrade().ok_or_else(|| {
        napi::Error::new(
          napi::Status::GenericFailure,
          "Failed to upgrade weak reference to heap",
        )
      })?;

      let inner = binding.inner();
      inner.jsobject.get_or_try_init(|| {
        if inner.data.type_id() == TypeId::of::<AssetInfo>() {
          ThreadsafeOneShotRef::new(env.raw(), object)
        } else {
          unreachable!()
        }
      })?;
      Ok(())
    }
  }

  impl<T: 'static> ToNapiValue for Reflector<T> {
    unsafe fn to_napi_value(
      raw_env: napi::sys::napi_env,
      val: Self,
    ) -> napi::Result<napi::sys::napi_value> {
      with_thread_local_allocator(|allocator| {
        let binding = val.upgrade().ok_or_else(|| {
          napi::Error::new(
            napi::Status::GenericFailure,
            "Failed to upgrade weak reference to BindingCell",
          )
        })?;

        let type_id = binding.as_ref().type_id();
        let raw_ref = binding.inner().jsobject.get_or_try_init(|| {
          if type_id == TypeId::of::<AssetInfo>() {
            let binding = BindingCell {
              ptr: binding.ptr.cast::<BindingCellInner<AssetInfo>>(),
            };
            let napi_val = allocator.allocate_asset_info(raw_env, &binding)?;
            let target = Object::from_raw(raw_env, napi_val);
            ThreadsafeOneShotRef::new(raw_env, target)
          } else if type_id == TypeId::of::<CodeGenerationResult>() {
            let binding = BindingCell {
              ptr: binding.ptr.cast::<BindingCellInner<CodeGenerationResult>>(),
            };
            let napi_val = allocator.allocate_code_generation_result(raw_env, &binding)?;
            let target = Object::from_raw(raw_env, napi_val);
            ThreadsafeOneShotRef::new(raw_env, target)
          } else if type_id == TypeId::of::<FxHashMap<SourceType, BoxSource>>() {
            let binding = BindingCell {
              ptr: binding
                .ptr
                .cast::<BindingCellInner<FxHashMap<SourceType, BoxSource>>>(),
            };
            let napi_val = allocator.allocate_sources(raw_env, &binding)?;
            let target = Object::from_raw(raw_env, napi_val);
            ThreadsafeOneShotRef::new(raw_env, target)
          } else if type_id == TypeId::of::<CodeGenerationResults>() {
            let binding = BindingCell {
              ptr: binding
                .ptr
                .cast::<BindingCellInner<CodeGenerationResults>>(),
            };
            let napi_val = allocator.allocate_code_generation_results(raw_env, &binding)?;
            let target = Object::from_raw(raw_env, napi_val);
            ThreadsafeOneShotRef::new(raw_env, target)
          } else if type_id == TypeId::of::<FxHashMap<String, CompilationAsset>>() {
            let binding = BindingCell {
              ptr: binding
                .ptr
                .cast::<BindingCellInner<FxHashMap<String, CompilationAsset>>>(),
            };
            let napi_val = allocator.allocate_assets(raw_env, &binding)?;
            let target = Object::from_raw(raw_env, napi_val);
            ThreadsafeOneShotRef::new(raw_env, target)
          } else {
            unreachable!()
          }
        })?;

        let result = unsafe { ToNapiValue::to_napi_value(raw_env, raw_ref)? };

        if type_id == TypeId::of::<AssetInfo>() {
          let binding = BindingCell {
            ptr: binding.ptr.cast::<BindingCellInner<AssetInfo>>(),
          };
          let napi_val = allocator.allocate_asset_info(raw_env, &binding)?;
          let new_object = Object::from_raw(raw_env, napi_val);
          let mut original_object = Object::from_raw(raw_env, result);
          object_assign(&mut original_object, &new_object)?;
        }
        Ok(result)
      })
    }
  }

  impl<T: ?Sized> Drop for Reflector<T> {
    fn drop(&mut self) {
      // If we find out that we were the last weak pointer, then its time to
      // deallocate the data entirely. See the discussion in BindingCell::drop() about
      // the memory orderings
      //
      // It's not necessary to check for the locked state here, because the
      // weak count can only be locked if there was precisely one weak ref,
      // meaning that drop could only subsequently run ON that remaining weak
      // ref, which can only happen after the lock is released.
      let inner = if let Some(inner) = self.inner() {
        inner
      } else {
        return;
      };

      if inner.weak.fetch_sub(1, Release) == 1 {
        inner.weak.load(Acquire);

        let raw_ptr: *mut BindingCellInner<T> = self.ptr.as_ptr();
        unsafe { std::alloc::dealloc(raw_ptr as *mut u8, Layout::for_value_raw(raw_ptr)) }
      }
    }
  }

  #[derive(Debug)]
  struct BindingCellInner<T: ?Sized + 'static> {
    strong: AtomicUsize,
    // the value usize::MAX acts as a sentinel for temporarily "locking" the
    // ability to upgrade weak pointers or downgrade strong ones; this is used
    // to avoid races in `make_mut` and `get_mut`.
    weak: AtomicUsize,
    #[debug(skip)]
    jsobject: OnceCell<ThreadsafeOneShotRef>,
    data: T,
  }

  #[derive(CoercePointee, Debug)]
  #[repr(transparent)]
  pub struct BindingCell<T: ?Sized + 'static> {
    ptr: NonNull<BindingCellInner<T>>,
  }

  impl<T: ?Sized> BindingCell<T> {
    /// Creates a new [`Reflector`] pointer to this allocation.
    pub fn reflector(&self) -> Reflector<T> {
      // This Relaxed is OK because we're checking the value in the CAS
      // below.
      let mut cur = self.inner().weak.load(Relaxed);

      loop {
        // check if the weak counter is currently "locked"; if so, spin.
        if cur == usize::MAX {
          hint::spin_loop();
          cur = self.inner().weak.load(Relaxed);
          continue;
        }

        // We can't allow the refcount to increase much past `MAX_REFCOUNT`.
        assert!(cur <= MAX_REFCOUNT, "{}", INTERNAL_OVERFLOW_ERROR);

        // NOTE: this code currently ignores the possibility of overflow
        // into usize::MAX; in general both Rc and BindingCell need to be adjusted
        // to deal with overflow.

        // Unlike with Clone(), we need this to be an Acquire read to
        // synchronize with the write coming from `is_unique`, so that the
        // events prior to that write happen before this read.
        match self
          .inner()
          .weak
          .compare_exchange_weak(cur, cur + 1, Acquire, Relaxed)
        {
          Ok(_) => {
            // Make sure we do not create a dangling Weak
            debug_assert!(!is_dangling(self.ptr.as_ptr()));
            return Reflector { ptr: self.ptr };
          }
          Err(old) => cur = old,
        }
      }
    }

    // Non-inlined part of `drop`.
    #[inline(never)]
    unsafe fn drop_slow(&mut self) {
      // Drop the weak ref collectively held by all strong references when this
      // variable goes out of scope. This ensures that the memory is deallocated
      // even if the destructor of `T` panics.
      // Take a reference to `self.alloc` instead of cloning because 1. it'll last long
      // enough, and 2. you should be able to drop `BindingCell`s with unclonable allocators
      let _weak = Reflector { ptr: self.ptr };

      // Destroy the data at this time, even though we must not free the box
      // allocation itself (there might still be weak pointers lying around).
      // We cannot use `get_mut_unchecked` here, because `self.alloc` is borrowed.
      unsafe { ptr::drop_in_place(&mut (*self.ptr.as_ptr()).data) };
    }

    #[inline]
    fn inner(&self) -> &BindingCellInner<T> {
      // This unsafety is ok because while this arc is alive we're guaranteed
      // that the inner pointer is valid. Furthermore, we know that the
      // `BindingCellInner` structure itself is `Sync` because the inner data is
      // `Sync` as well, so we're ok loaning out an immutable pointer to these
      // contents.
      unsafe { self.ptr.as_ref() }
    }

    #[inline]
    fn inner_mut(&mut self) -> &mut BindingCellInner<T> {
      // This unsafety is ok because while this arc is alive we're guaranteed
      // that the inner pointer is valid. Furthermore, we know that the
      // `BindingCellInner` structure itself is `Sync` because the inner data is
      // `Sync` as well, so we're ok loaning out an immutable pointer to these
      // contents.
      unsafe { self.ptr.as_mut() }
    }

    #[inline]
    unsafe fn from_inner(ptr: NonNull<BindingCellInner<T>>) -> Self {
      Self { ptr }
    }
  }

  unsafe impl<T: ?Sized + Send> Send for BindingCell<T> {}
  unsafe impl<T: ?Sized + Sync> Sync for BindingCell<T> {}

  impl<T> BindingCell<T> {
    pub fn new(data: T) -> Self {
      // Start the weak pointer count as 1 which is the weak pointer that's
      // held by all the strong pointers (kinda), see std/rc.rs for more info
      let x = Box::new(BindingCellInner {
        strong: AtomicUsize::new(1),
        weak: AtomicUsize::new(1),
        jsobject: OnceCell::default(),
        data,
      });
      unsafe { Self::from_inner(Box::leak(x).into()) }
    }
  }

  impl<T> From<T> for BindingCell<T> {
    fn from(value: T) -> Self {
      Self::new(value)
    }
  }

  impl<T: ?Sized> AsRef<T> for BindingCell<T> {
    fn as_ref(&self) -> &T {
      &self.inner().data
    }
  }

  impl<T: ?Sized> AsMut<T> for BindingCell<T> {
    fn as_mut(&mut self) -> &mut T {
      &mut self.inner_mut().data
    }
  }

  impl<T: ?Sized> Deref for BindingCell<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
      &self.inner().data
    }
  }

  impl<T: ?Sized> DerefMut for BindingCell<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
      &mut self.inner_mut().data
    }
  }

  impl<T: Clone> Clone for BindingCell<T> {
    fn clone(&self) -> Self {
      let val = self.as_ref().clone();
      Self::new(val)
    }
  }

  impl<T: ?Sized + Hash> Hash for BindingCell<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
      self.as_ref().hash(state);
    }
  }

  impl<T: ?Sized + PartialEq> PartialEq for BindingCell<T> {
    fn eq(&self, other: &Self) -> bool {
      self.as_ref() == other.as_ref()
    }
  }

  impl<T: ?Sized + Eq> Eq for BindingCell<T> {}

  impl<T: Default> Default for BindingCell<T> {
    fn default() -> Self {
      let value: T = Default::default();
      Self::new(value)
    }
  }

  impl<T: ?Sized> Drop for BindingCell<T> {
    #[inline]
    fn drop(&mut self) {
      // Because `fetch_sub` is already atomic, we do not need to synchronize
      // with other threads unless we are going to delete the object. This
      // same logic applies to the below `fetch_sub` to the `weak` count.
      if self.inner().strong.fetch_sub(1, Release) != 1 {
        return;
      }

      // This fence is needed to prevent reordering of use of the data and
      // deletion of the data. Because it is marked `Release`, the decreasing
      // of the reference count synchronizes with this `Acquire` fence. This
      // means that use of the data happens before decreasing the reference
      // count, which happens before this fence, which happens before the
      // deletion of the data.
      //
      // As explained in the [Boost documentation][1],
      //
      // > It is important to enforce any possible access to the object in one
      // > thread (through an existing reference) to *happen before* deleting
      // > the object in a different thread. This is achieved by a "release"
      // > operation after dropping a reference (any access to the object
      // > through this reference must obviously happened before), and an
      // > "acquire" operation before deleting the object.
      //
      // In particular, while the contents of an BindingCell are usually immutable, it's
      // possible to have interior writes to something like a Mutex<T>. Since a
      // Mutex is not acquired when it is deleted, we can't rely on its
      // synchronization logic to make writes in thread A visible to a destructor
      // running in thread B.
      //
      // Also note that the Acquire fence here could probably be replaced with an
      // Acquire load, which could improve performance in highly-contended
      // situations. See [2].
      //
      // [1]: (www.boost.org/doc/libs/1_55_0/doc/html/atomic/usage_examples.html)
      // [2]: (https://github.com/rust-lang/rust/pull/41714)
      self.inner().strong.load(Acquire);

      unsafe {
        self.drop_slow();
      }
    }
  }

  // Implement rkyv traits for BindingCell<T>

  impl<T: rkyv::ArchiveUnsized + ?Sized> rkyv::Archive for BindingCell<T> {
    type Archived = rkyv::boxed::ArchivedBox<T::Archived>;
    type Resolver = rkyv::boxed::BoxResolver;

    fn resolve(&self, resolver: Self::Resolver, out: rkyv::Place<Self::Archived>) {
      rkyv::boxed::ArchivedBox::resolve_from_ref(self.as_ref(), resolver, out);
    }
  }

  impl<T, S> rkyv::Serialize<S> for BindingCell<T>
  where
    T: rkyv::SerializeUnsized<S> + ?Sized,
    S: rkyv::rancor::Fallible + ?Sized,
  {
    fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
      rkyv::boxed::ArchivedBox::serialize_from_ref(self.as_ref(), serializer)
    }
  }

  impl<T, D> rkyv::Deserialize<BindingCell<T>, D> for rkyv::boxed::ArchivedBox<T::Archived>
  where
    T: rkyv::ArchiveUnsized + rkyv::traits::LayoutRaw + ?Sized + 'static,
    T::Archived: rkyv::DeserializeUnsized<T, D>,
    D: rkyv::rancor::Fallible + ?Sized,
    D::Error: rkyv::rancor::Source,
  {
    fn deserialize(&self, deserializer: &mut D) -> Result<BindingCell<T>, D::Error> {
      // let boxed: Box<T> = self.deserialize(deserializer)?;
      // Ok(BindingCell::new(*boxed))
      todo!()
    }
  }
}

#[cfg(not(feature = "napi"))]
mod sys_binding {
  pub type BindingCell<T> = Box<T>;
}

#[cfg(feature = "napi")]
pub use napi_binding::*;
#[cfg(not(feature = "napi"))]
pub use sys_binding::*;
