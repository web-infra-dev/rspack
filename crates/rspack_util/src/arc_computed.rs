use std::{ops::Deref, sync::Arc};

use rspack_cacheable::with::AsInnerConverter;

#[derive(Debug)]
pub struct ArcComputed<T, U> {
  owner: Arc<T>,
  computed: *const U,
}

impl<T, U> ArcComputed<T, U> {
  pub fn new(owner: Arc<T>, compute: impl FnOnce(&T) -> &U) -> Self {
    let computed = compute(&owner) as *const U;
    Self { owner, computed }
  }

  pub fn try_new(owner: Arc<T>, compute: impl FnOnce(&T) -> Option<&U>) -> Option<Self> {
    let computed = compute(&owner)? as *const U;
    Some(Self { owner, computed })
  }
}

impl<T, U> Clone for ArcComputed<T, U> {
  fn clone(&self) -> Self {
    Self {
      owner: Arc::clone(&self.owner),
      computed: self.computed,
    }
  }
}

impl<T, U> AsInnerConverter for ArcComputed<T, U>
where
  for<'a> &'a U: From<&'a T>,
{
  type Inner = Arc<T>;

  fn to_inner(&self) -> &Self::Inner {
    &self.owner
  }

  fn from_inner(data: Self::Inner) -> Self {
    Self::new(data, |owner| owner.into())
  }
}

impl<T, U> Deref for ArcComputed<T, U> {
  type Target = U;

  fn deref(&self) -> &Self::Target {
    // SAFETY: `computed` is created from a shared reference into `owner`.
    // `owner` is kept alive by this struct and the computed value is immutable.
    unsafe { &*self.computed }
  }
}

impl<T, U> AsRef<U> for ArcComputed<T, U> {
  fn as_ref(&self) -> &U {
    self
  }
}

unsafe impl<T, U> Send for ArcComputed<T, U>
where
  Arc<T>: Send,
  U: Sync,
{
}

unsafe impl<T, U> Sync for ArcComputed<T, U>
where
  Arc<T>: Sync,
  U: Sync,
{
}
