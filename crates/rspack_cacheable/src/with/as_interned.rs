use std::sync::Arc;

use rkyv::{
  Place,
  de::Pooling,
  rancor::Fallible,
  string::ArchivedString,
  with::{ArchiveWith, DeserializeWith, SerializeWith},
};

use crate::{ContextGuard, Error, Result};

/// Serializes `Arc<T>` through `A` and restores equal values as one shared `Arc`.
///
/// Each cache entry is decoded on its own, so rkyv's sharing cannot reach across entries. Use this
/// for values held by many entries. The archived text is the key, so entries holding different
/// content never alias, and only the first occurrence pays for parsing.
///
/// Values are shared only while the context provides an [`Interner`](crate::Interner); otherwise
/// each decode gets its own `Arc`. `T` must not rely on `Arc` identity or be mutated in place.
pub struct AsInterned<A> {
  _inner: A,
}

impl<T, A> ArchiveWith<Arc<T>> for AsInterned<A>
where
  A: ArchiveWith<T, Archived = ArchivedString>,
{
  type Archived = ArchivedString;
  type Resolver = A::Resolver;

  #[inline]
  fn resolve_with(field: &Arc<T>, resolver: Self::Resolver, out: Place<Self::Archived>) {
    A::resolve_with(field, resolver, out)
  }
}

impl<T, A, S> SerializeWith<Arc<T>, S> for AsInterned<A>
where
  A: ArchiveWith<T, Archived = ArchivedString> + SerializeWith<T, S>,
  S: Fallible + ?Sized,
{
  #[inline]
  fn serialize_with(field: &Arc<T>, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
    A::serialize_with(field, serializer)
  }
}

impl<T, A, D> DeserializeWith<ArchivedString, Arc<T>, D> for AsInterned<A>
where
  T: Send + Sync + 'static,
  A: ArchiveWith<T, Archived = ArchivedString> + DeserializeWith<ArchivedString, T, D>,
  D: Fallible<Error = Error> + Pooling + ?Sized,
{
  fn deserialize_with(field: &ArchivedString, de: &mut D) -> Result<Arc<T>> {
    // The guard borrows `de`, so it can't be held across the inner deserialization.
    if let Some(interner) = ContextGuard::pooling_guard(de)?.interner()
      && let Some(value) = interner.get::<T>(field)
    {
      return Ok(value);
    }

    let value = Arc::new(A::deserialize_with(field, de)?);
    Ok(match ContextGuard::pooling_guard(de)?.interner() {
      Some(interner) => interner.insert(field, value),
      None => value,
    })
  }
}
