use std::any::Any;

use rkyv::{
  Archive, Deserialize, Place, Serialize,
  de::Pooling,
  rancor::Fallible,
  ser::Sharing,
  with::{ArchiveWith, DeserializeWith, SerializeWith},
};

use crate::{Error, Result, cacheable, context::ContextGuard};

/// A trait for writing custom serialization and deserialization.
///
/// `#[cacheable(with=Custom)]` will use this trait.
pub trait CustomConverter {
  type Target: Archive;
  fn serialize(&self, ctx: &dyn Any) -> Result<Self::Target>;
  fn deserialize(data: Self::Target, ctx: &dyn Any) -> Result<Self>
  where
    Self: Sized;
}

/// A wrapper that uses CustomConverter for serialization.
pub struct Custom;

/// A simple structure to save the generated `CustomConverter::Target`,
/// which can avoid some deserialization conflicts.
#[cacheable(crate=crate)]
pub struct DataBox<T: Archive>(T);

pub struct CustomResolver<A: Archive> {
  resolver: DataBoxResolver<A>,
  value: DataBox<A>,
}

impl<T> ArchiveWith<T> for Custom
where
  T: CustomConverter,
  T::Target: Archive,
{
  type Archived = ArchivedDataBox<T::Target>;
  type Resolver = CustomResolver<T::Target>;

  #[inline]
  fn resolve_with(_field: &T, resolver: Self::Resolver, out: Place<Self::Archived>) {
    let CustomResolver { resolver, value } = resolver;
    value.resolve(resolver, out)
  }
}

impl<T, S> SerializeWith<T, S> for Custom
where
  T: CustomConverter,
  T::Target: Archive + Serialize<S>,
  S: Fallible<Error = Error> + Sharing + ?Sized,
{
  #[inline]
  fn serialize_with(field: &T, serializer: &mut S) -> Result<Self::Resolver> {
    let ctx = ContextGuard::sharing_context(serializer)?;
    let value = DataBox(T::serialize(field, ctx)?);
    Ok(CustomResolver {
      resolver: value.serialize(serializer)?,
      value,
    })
  }
}

impl<T, D> DeserializeWith<ArchivedDataBox<T::Target>, T, D> for Custom
where
  T: CustomConverter,
  T::Target: Archive,
  ArchivedDataBox<T::Target>: Deserialize<DataBox<T::Target>, D>,
  D: Fallible<Error = Error> + Pooling + ?Sized,
{
  #[inline]
  fn deserialize_with(field: &ArchivedDataBox<T::Target>, de: &mut D) -> Result<T> {
    let value = field.deserialize(de)?;
    let ctx = ContextGuard::pooling_context(de)?;
    T::deserialize(value.0, ctx)
  }
}
