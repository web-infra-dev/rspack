use rkyv::{
  Archive, Archived, Deserialize, Place, Resolver, Serialize,
  de::Pooling,
  rancor::Fallible,
  ser::Sharing,
  with::{ArchiveWith, DeserializeWith, SerializeWith},
};

use crate::{Error, Result, context::ContextGuard};

pub trait AsConverter<T> {
  fn serialize(data: &T, ctx: &ContextGuard) -> Result<Self>
  where
    Self: Sized;
  fn deserialize(self, ctx: &ContextGuard) -> Result<T>;
}

pub struct As<A> {
  _inner: A,
}

pub struct AsResolver<A: Archive> {
  resolver: Resolver<A>,
  value: A,
}

impl<T, A> ArchiveWith<T> for As<A>
where
  A: AsConverter<T> + Archive,
{
  type Archived = Archived<A>;
  type Resolver = AsResolver<A>;

  #[inline]
  fn resolve_with(_field: &T, resolver: Self::Resolver, out: Place<Self::Archived>) {
    let AsResolver { resolver, value } = resolver;
    value.resolve(resolver, out)
  }
}

impl<T, A, S> SerializeWith<T, S> for As<A>
where
  A: AsConverter<T> + Archive + Serialize<S>,
  S: Fallible<Error = Error> + Sharing + ?Sized,
{
  #[inline]
  fn serialize_with(field: &T, serializer: &mut S) -> Result<Self::Resolver> {
    let guard = ContextGuard::sharing_guard(serializer)?;
    let value = <A as AsConverter<T>>::serialize(field, guard)?;
    Ok(AsResolver {
      resolver: value.serialize(serializer)?,
      value,
    })
  }
}

impl<T, A, D> DeserializeWith<Archived<A>, T, D> for As<A>
where
  A: AsConverter<T> + Archive,
  A::Archived: Deserialize<A, D>,
  D: Fallible<Error = Error> + Pooling + ?Sized,
{
  #[inline]
  fn deserialize_with(field: &Archived<A>, de: &mut D) -> Result<T> {
    let field = A::Archived::deserialize(field, de)?;
    let guard = ContextGuard::pooling_guard(de)?;
    field.deserialize(guard)
  }
}
