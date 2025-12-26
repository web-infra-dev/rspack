use std::any::Any;

use rkyv::{
  Archive, Archived, Deserialize, Place, Resolver, Serialize,
  de::Pooling,
  rancor::Fallible,
  ser::Sharing,
  with::{ArchiveWith, DeserializeWith, SerializeWith},
};

use crate::{Error, Result, context::ContextGuard};

pub trait AsConverter<T> {
  fn serialize(data: &T, ctx: &dyn Any) -> Result<Self>
  where
    Self: Sized;
  fn deserialize(self, ctx: &dyn Any) -> Result<T>;
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
    let ctx = ContextGuard::sharing_context(serializer)?;
    let value = <A as AsConverter<T>>::serialize(field, ctx)?;
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
    let ctx = ContextGuard::pooling_context(de)?;
    field.deserialize(ctx)
  }
}
