use rkyv::{
  Archive, Archived, Deserialize, Place, Resolver, Serialize,
  rancor::Fallible,
  ser::{Allocator, Writer},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
};
use rspack_macros::enable_cacheable as cacheable;
use rspack_sources::{
  BoxSource, ObjectPool, RawBufferSource, Source, SourceExt, SourceMap, SourceMapSource,
  WithoutOriginalOptions,
};

use super::AsPreset;
use crate::{DeserializeError, SerializeError};

#[cacheable(crate=crate)]
pub struct CacheableSource {
  buffer: Vec<u8>,
  map: Option<String>,
}

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
  S: Fallible<Error = SerializeError> + Allocator + Writer,
{
  fn serialize_with(
    field: &BoxSource,
    serializer: &mut S,
  ) -> Result<Self::Resolver, SerializeError> {
    let map = match field.map(&ObjectPool::default(), &Default::default()) {
      Some(map) => Some(
        map
          .to_json()
          .map_err(|_| SerializeError::MessageError("source map to json failed"))?,
      ),
      None => None,
    };
    let source = CacheableSource {
      buffer: field.buffer().to_vec(),
      map,
    };
    Ok(InnerResolver {
      resolver: source.serialize(serializer)?,
      source,
    })
  }
}

impl<D> DeserializeWith<Archived<CacheableSource>, BoxSource, D> for AsPreset
where
  D: Fallible<Error = DeserializeError>,
{
  fn deserialize_with(
    field: &Archived<CacheableSource>,
    deserializer: &mut D,
  ) -> Result<BoxSource, DeserializeError> {
    let CacheableSource { buffer, map } = field.deserialize(deserializer)?;
    if let Some(map) = &map
      && let Ok(source_map) = SourceMap::from_json(map)
    {
      return Ok(
        SourceMapSource::new(WithoutOriginalOptions {
          value: String::from_utf8_lossy(&buffer),
          name: "persistent-cache",
          source_map,
        })
        .boxed(),
      );
    }
    Ok(RawBufferSource::from(buffer).boxed())
  }
}
