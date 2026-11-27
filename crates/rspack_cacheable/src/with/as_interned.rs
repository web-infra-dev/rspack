use std::{
  any::{Any, TypeId},
  hash::BuildHasherDefault,
  sync::{
    Arc, LazyLock, Weak,
    atomic::{AtomicUsize, Ordering},
  },
};

use dashmap::{DashMap, mapref::entry::Entry};
use rkyv::{
  Place,
  rancor::Fallible,
  string::ArchivedString,
  with::{ArchiveWith, DeserializeWith, SerializeWith},
};
use rustc_hash::FxHasher;
use xxhash_rust::xxh3::xxh3_128;

use crate::with::AsCacheable;

pub struct AsInterned<T = AsCacheable> {
  _inner: T,
}

pub trait AsInternedConverter {
  type Inner: Send + Sync + 'static;
  fn to_inner(&self) -> &Self::Inner;
  fn from_interned(inner: Arc<Self::Inner>) -> Self;
}

impl<T: Send + Sync + 'static> AsInternedConverter for Arc<T> {
  type Inner = T;
  fn to_inner(&self) -> &Self::Inner {
    self
  }
  fn from_interned(inner: Arc<Self::Inner>) -> Self {
    inner
  }
}

type InternKey = (TypeId, u128);
type InternedValue = Weak<dyn Any + Send + Sync>;

const MIN_PRUNE_THRESHOLD: usize = 1024;

static INTERNED_VALUES: LazyLock<DashMap<InternKey, InternedValue, BuildHasherDefault<FxHasher>>> =
  LazyLock::new(Default::default);
static PRUNE_THRESHOLD: AtomicUsize = AtomicUsize::new(MIN_PRUNE_THRESHOLD);

fn get_interned<T: Send + Sync + 'static>(key: &InternKey) -> Option<Arc<T>> {
  INTERNED_VALUES.get(key)?.upgrade()?.downcast().ok()
}

fn intern<T: Send + Sync + 'static>(key: InternKey, value: T) -> Arc<T> {
  let value = Arc::new(value);
  match INTERNED_VALUES.entry(key) {
    Entry::Occupied(mut entry) => {
      if let Some(existing) = entry.get().upgrade().and_then(|v| v.downcast().ok()) {
        return existing;
      }
      entry.insert(Arc::downgrade(&value) as InternedValue);
      return value;
    }
    Entry::Vacant(entry) => {
      entry.insert(Arc::downgrade(&value) as InternedValue);
    }
  }
  prune_dead_values();
  value
}

fn prune_dead_values() {
  if INTERNED_VALUES.len() < PRUNE_THRESHOLD.load(Ordering::Relaxed) {
    return;
  }
  INTERNED_VALUES.retain(|_, value| value.strong_count() > 0);
  PRUNE_THRESHOLD.store(
    (INTERNED_VALUES.len() * 2).max(MIN_PRUNE_THRESHOLD),
    Ordering::Relaxed,
  );
}

impl<T, A> ArchiveWith<T> for AsInterned<A>
where
  T: AsInternedConverter,
  A: ArchiveWith<T::Inner, Archived = ArchivedString>,
{
  type Archived = ArchivedString;
  type Resolver = A::Resolver;

  #[inline]
  fn resolve_with(field: &T, resolver: Self::Resolver, out: Place<Self::Archived>) {
    A::resolve_with(field.to_inner(), resolver, out)
  }
}

impl<T, A, S> SerializeWith<T, S> for AsInterned<A>
where
  T: AsInternedConverter,
  A: ArchiveWith<T::Inner, Archived = ArchivedString> + SerializeWith<T::Inner, S>,
  S: Fallible + ?Sized,
{
  #[inline]
  fn serialize_with(field: &T, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
    A::serialize_with(field.to_inner(), serializer)
  }
}

impl<T, A, D> DeserializeWith<ArchivedString, T, D> for AsInterned<A>
where
  T: AsInternedConverter,
  A: ArchiveWith<T::Inner, Archived = ArchivedString>
    + DeserializeWith<ArchivedString, T::Inner, D>
    + 'static,
  D: Fallible + ?Sized,
{
  fn deserialize_with(field: &ArchivedString, deserializer: &mut D) -> Result<T, D::Error> {
    let key = (
      TypeId::of::<(A, T::Inner)>(),
      xxh3_128(field.as_str().as_bytes()),
    );
    if let Some(interned) = get_interned(&key) {
      return Ok(T::from_interned(interned));
    }
    let value = A::deserialize_with(field, deserializer)?;
    Ok(T::from_interned(intern(key, value)))
  }
}
