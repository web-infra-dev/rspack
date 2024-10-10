use std::any::Any;

use rkyv::{
  with::{ArchiveWith, DeserializeWith, SerializeWith},
  Archive, Archived, Deserialize, Place, Resolver, Serialize,
};

use crate::{
  deserialize::get_deserializer_context, serialize::get_serializer_context, DeserializeError,
  Deserializer, SerializeError, Serializer,
};

pub trait AsConverter<T> {
  type Context;
  fn deserialize(&self, ctx: &Self::Context) -> Result<T, DeserializeError>;
  fn serialize(data: &T, ctx: &Self::Context) -> Result<Self, SerializeError>
  where
    Self: Sized;
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

impl<T, A> SerializeWith<T, Serializer<'_>> for As<A>
where
  A: AsConverter<T> + Archive + for<'a> Serialize<Serializer<'a>>,
  A::Context: Any,
{
  #[inline]
  fn serialize_with(
    field: &T,
    serializer: &mut Serializer,
  ) -> Result<Self::Resolver, SerializeError> {
    let Some(ctx) = get_serializer_context::<A::Context>(serializer) else {
      return Err(SerializeError::GetContextFailed);
    };
    let value = <A as AsConverter<T>>::serialize(field, ctx)?;
    Ok(AsResolver {
      resolver: value.serialize(serializer)?,
      value,
    })
  }
}

impl<T, A> DeserializeWith<Archived<A>, T, Deserializer> for As<A>
where
  A: AsConverter<T> + Archive,
  A::Archived: Deserialize<A, Deserializer>,
  A::Context: Any,
{
  #[inline]
  fn deserialize_with(field: &Archived<A>, de: &mut Deserializer) -> Result<T, DeserializeError> {
    let field = A::Archived::deserialize(field, de)?;
    let Some(ctx) = get_deserializer_context::<A::Context>(de) else {
      return Err(DeserializeError::GetContextFailed);
    };
    field.deserialize(ctx)
  }
}
