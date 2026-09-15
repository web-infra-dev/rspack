use std::{
  any::TypeId,
  fmt,
  hash::{Hash, Hasher},
  marker::PhantomData,
  sync::atomic::{AtomicU64, Ordering},
};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// A process-unique identity, typed independently of the owner's generic parameters.
pub struct LifecycleId<T: 'static> {
  pub(crate) value: u64,
  marker: PhantomData<fn() -> T>,
}

impl<T> Copy for LifecycleId<T> {}
impl<T> Clone for LifecycleId<T> {
  fn clone(&self) -> Self {
    *self
  }
}
impl<T> PartialEq for LifecycleId<T> {
  fn eq(&self, other: &Self) -> bool {
    self.value == other.value
  }
}
impl<T> Eq for LifecycleId<T> {}
impl<T> Hash for LifecycleId<T> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.value.hash(state);
  }
}
impl<T> fmt::Debug for LifecycleId<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.value.fmt(f)
  }
}

/// Owns an identity without owning or borrowing `T`. Moving the guard preserves
/// its identity; dropping it releases associated JS references on their threads.
/// This guard deliberately cannot be cloned.
pub struct LifecycleGuard<T: 'static> {
  id: LifecycleId<T>,
}

impl<T> LifecycleGuard<T> {
  pub fn new() -> Self {
    let value = NEXT_ID
      .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
      .expect("Lifecycle IDs exhausted");
    Self {
      id: LifecycleId {
        value,
        marker: PhantomData,
      },
    }
  }

  pub fn id(&self) -> LifecycleId<T> {
    self.id
  }
}

impl<T> Default for LifecycleGuard<T> {
  fn default() -> Self {
    Self::new()
  }
}

impl<T> fmt::Debug for LifecycleGuard<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.id.fmt(f)
  }
}

impl<T> Drop for LifecycleGuard<T> {
  fn drop(&mut self) {
    crate::thread_local_reference::notify_drop(TypeId::of::<T>(), self.id.value);
  }
}
