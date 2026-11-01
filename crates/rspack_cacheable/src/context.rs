use std::{
  any::{Any, TypeId},
  hash::BuildHasherDefault,
  ops::Deref,
  path::Path,
  sync::Arc,
};

use dashmap::DashMap;
use rkyv::{
  de::{ErasedPtr, Pooling, PoolingState},
  ser::{Sharing, sharing::SharingState},
};
use rustc_hash::FxHasher;

use crate::error::{Error, Result};

const CONTEXT_ADDR: usize = 0;
unsafe fn default_drop(_: ErasedPtr) {}

type FxDashMap<K, V> = DashMap<K, V, BuildHasherDefault<FxHasher>>;

/// Shares equal deserialized values across `from_bytes` calls.
///
/// Entries live as long as the interner, so tie it to the owner of the decoded data rather than to
/// a process-wide static. Values are grouped by type so that lookups can borrow the key: the keys
/// may be whole `package.json` files, and a hit must not copy them.
#[derive(Debug, Default)]
pub struct Interner {
  types: FxDashMap<TypeId, FxDashMap<Box<str>, Arc<dyn Any + Send + Sync>>>,
}

impl Interner {
  /// Returns the value interned under `key`.
  pub fn get<T: Send + Sync + 'static>(&self, key: &str) -> Option<Arc<T>> {
    let values = self.types.get(&TypeId::of::<T>())?;
    let value = Arc::clone(values.get(key)?.value());
    value.downcast().ok()
  }

  /// Interns `value` under `key` and returns the shared value, which is an earlier one when
  /// another decode got there first.
  pub fn insert<T: Send + Sync + 'static>(&self, key: &str, value: Arc<T>) -> Arc<T> {
    let values = self.types.entry(TypeId::of::<T>()).or_default();
    let shared = Arc::clone(values.entry(key.into()).or_insert(value).value());
    shared
      .downcast()
      .expect("interned values are grouped by their type")
  }
}

pub trait CacheableContext: Any {
  /// Returns the project root used for portable path conversion.
  ///
  /// `Some` enables portable conversion relative to this root, while `None` keeps paths in their
  /// native representation.
  fn project_root(&self) -> Option<&Path>;

  /// Returns the interner backing [`AsInterned`](crate::with::AsInterned) fields.
  ///
  /// `None` restores those fields without sharing.
  fn interner(&self) -> Option<&Interner> {
    None
  }
}

// Implement for unit type for convenience in tests and simple cases
impl CacheableContext for () {
  fn project_root(&self) -> Option<&Path> {
    None
  }
}

/// A context wrapper that provides shared context methods
pub struct ContextGuard<'a> {
  context: &'a dyn CacheableContext,
}

impl<'a> Deref for ContextGuard<'a> {
  type Target = dyn CacheableContext;
  fn deref(&self) -> &Self::Target {
    self.context
  }
}

impl<'a> ContextGuard<'a> {
  pub fn new(context: &'a dyn CacheableContext) -> Self {
    Self { context }
  }

  pub fn add_to_sharing<S: Sharing<Error> + ?Sized>(&self, sharing: &mut S) -> Result<()> {
    sharing.start_sharing(CONTEXT_ADDR);
    sharing.finish_sharing(CONTEXT_ADDR, self as *const _ as usize)
  }

  pub fn sharing_guard<S: Sharing<Error> + ?Sized>(sharing: &'a mut S) -> Result<&'a Self> {
    match sharing.start_sharing(CONTEXT_ADDR) {
      SharingState::Finished(addr) => {
        let guard: &Self = unsafe { &*(addr as *const Self) };
        Ok(guard)
      }
      _ => Err(Error::NoContext),
    }
  }

  pub fn add_to_pooling<P: Pooling<Error> + ?Sized>(&self, pooling: &mut P) -> Result<()> {
    unsafe {
      let ctx_ptr = ErasedPtr::new(self as *const _ as *mut ());
      pooling.start_pooling(CONTEXT_ADDR);
      pooling.finish_pooling(CONTEXT_ADDR, ctx_ptr, default_drop)
    }
  }

  pub fn pooling_guard<P: Pooling<Error> + ?Sized>(pooling: &'a mut P) -> Result<&'a Self> {
    match pooling.start_pooling(CONTEXT_ADDR) {
      PoolingState::Finished(ptr) => {
        let guard: &Self = unsafe { &*(ptr.data_address() as *const Self) };
        Ok(guard)
      }
      _ => Err(Error::NoContext),
    }
  }

  pub fn downcast_context<T: 'static>(&'a self) -> Result<&'a T> {
    (self.context as &dyn Any)
      .downcast_ref()
      .ok_or(Error::NoContext)
  }
}
