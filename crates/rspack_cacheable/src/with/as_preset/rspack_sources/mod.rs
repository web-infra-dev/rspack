use rkyv::{
  Archive, Archived, Deserialize, Place, Resolver, Serialize,
  rancor::Fallible,
  ser::{Allocator, Writer},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
};
use rspack_sources::{
  BoxSource,
  cacheable::{CacheableSource, from_cacheable, to_cacheable},
};

use super::AsPreset;
use crate::{Error, Result};

pub struct InnerResolver {
  source: CacheableSource,
  resolver: Resolver<CacheableSource>,
}

impl ArchiveWith<BoxSource> for AsPreset {
  type Archived = Archived<CacheableSource>;
  type Resolver = InnerResolver;

  #[inline]
  fn resolve_with(_field: &BoxSource, resolver: Self::Resolver, out: Place<Self::Archived>) {
    let InnerResolver { source, resolver } = resolver;
    source.resolve(resolver, out)
  }
}

impl<S> SerializeWith<BoxSource, S> for AsPreset
where
  S: Fallible<Error = Error> + Allocator + Writer,
{
  fn serialize_with(field: &BoxSource, serializer: &mut S) -> Result<Self::Resolver> {
    let source = to_cacheable(field.as_ref());
    Ok(InnerResolver {
      resolver: source.serialize(serializer)?,
      source,
    })
  }
}

impl<D> DeserializeWith<Archived<CacheableSource>, BoxSource, D> for AsPreset
where
  D: Fallible<Error = Error>,
{
  fn deserialize_with(
    field: &Archived<CacheableSource>,
    deserializer: &mut D,
  ) -> Result<BoxSource> {
    let cacheable: CacheableSource = field.deserialize(deserializer)?;
    Ok(from_cacheable(cacheable))
  }
}
