use rkyv::{
  Archive, Archived, Deserialize, Place, Resolver, Serialize,
  rancor::Fallible,
  ser::{Allocator, Writer},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
};
use rspack_cacheable_macros::enable_cacheable as cacheable;
use rspack_sources::{
  BoxSource, CachedSource, ConcatSource, OriginalSource, RawBufferSource, RawStringSource,
  ReplaceSource, ReplacementEnforce, Source, SourceExt, SourceMap, SourceMapSource,
  SourceMapSourceOptions,
};

use super::AsPreset;
use crate::{Error, Result};

#[cacheable(crate=crate)]
pub struct CacheableReplacement {
  start: u32,
  end: u32,
  content: String,
  name: Option<String>,
  enforce: u8, // 0 = Pre, 1 = Normal, 2 = Post
}

#[cacheable(crate=crate)]
pub enum CacheableSource {
  RawBuffer {
    buffer: Vec<u8>,
  },
  RawString {
    value: String,
  },
  Original {
    value: String,
    name: String,
  },
  SourceMap {
    value: String,
    name: String,
    source_map: String,
    original_source: Option<String>,
    inner_source_map: Option<String>,
    remove_original_source: bool,
  },
  Concat {
    #[cacheable(omit_bounds)]
    children: Vec<CacheableSource>,
  },
  Replace {
    #[cacheable(omit_bounds)]
    inner: Box<CacheableSource>,
    replacements: Vec<CacheableReplacement>,
  },
  Cached {
    #[cacheable(omit_bounds)]
    inner: Box<CacheableSource>,
  },
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

fn to_cacheable(source: &dyn Source) -> CacheableSource {
  if let Some(s) = source.as_any().downcast_ref::<CachedSource>() {
    return CacheableSource::Cached {
      inner: Box::new(to_cacheable(s.inner().as_ref())),
    };
  }

  if let Some(s) = source.as_any().downcast_ref::<OriginalSource>() {
    return CacheableSource::Original {
      value: s.value().to_string(),
      name: s.name().to_string(),
    };
  }

  if let Some(s) = source.as_any().downcast_ref::<RawStringSource>() {
    return CacheableSource::RawString {
      value: s.source().into_string_lossy().into_owned(),
    };
  }

  if let Some(s) = source.as_any().downcast_ref::<RawBufferSource>() {
    return CacheableSource::RawBuffer {
      buffer: s.buffer().into_owned(),
    };
  }

  if let Some(s) = source.as_any().downcast_ref::<SourceMapSource>() {
    return CacheableSource::SourceMap {
      value: s.value().to_string(),
      name: s.name().to_string(),
      source_map: s.source_map().to_json(),
      original_source: s.original_source().map(|v| v.to_string()),
      inner_source_map: s.inner_source_map().map(|m| m.to_json()),
      remove_original_source: s.remove_original_source(),
    };
  }

  if let Some(s) = source.as_any().downcast_ref::<ConcatSource>() {
    return CacheableSource::Concat {
      children: s
        .children()
        .iter()
        .map(|c| to_cacheable(c.as_ref()))
        .collect(),
    };
  }

  if let Some(s) = source.as_any().downcast_ref::<ReplaceSource>() {
    let replacements = s
      .replacements()
      .iter()
      .map(|r| CacheableReplacement {
        start: r.start(),
        end: r.end(),
        content: r.content().to_string(),
        name: r.name().map(|n| n.to_string()),
        enforce: match r.enforce() {
          ReplacementEnforce::Pre => 0,
          ReplacementEnforce::Normal => 1,
          ReplacementEnforce::Post => 2,
        },
      })
      .collect();
    return CacheableSource::Replace {
      inner: Box::new(to_cacheable(s.inner().as_ref())),
      replacements,
    };
  }

  panic!(
    "Unexpected source type in persistent cache serialization. All BoxSource instances should be one of the known rspack_sources types."
  )
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

fn from_cacheable(cacheable: CacheableSource) -> BoxSource {
  match cacheable {
    CacheableSource::RawBuffer { buffer } => RawBufferSource::from(buffer).boxed(),
    CacheableSource::RawString { value } => RawStringSource::from(value).boxed(),
    CacheableSource::Original { value, name } => OriginalSource::new(value, name).boxed(),
    CacheableSource::SourceMap {
      value,
      name,
      source_map,
      original_source,
      inner_source_map,
      remove_original_source,
    } => {
      let source_map = SourceMap::from_json(&source_map).expect("invalid source map JSON");
      let inner_source_map = inner_source_map.and_then(|json| SourceMap::from_json(&json).ok());
      SourceMapSource::new(SourceMapSourceOptions {
        value,
        name,
        source_map,
        original_source: original_source.map(|s| s.into()),
        inner_source_map,
        remove_original_source,
      })
      .boxed()
    }
    CacheableSource::Concat { children } => {
      let children: Vec<BoxSource> = children.into_iter().map(from_cacheable).collect();
      ConcatSource::new(children).boxed()
    }
    CacheableSource::Replace {
      inner,
      replacements,
    } => {
      let inner = from_cacheable(*inner);
      let mut source = ReplaceSource::new(inner);
      for r in replacements {
        let enforce = match r.enforce {
          0 => ReplacementEnforce::Pre,
          1 => ReplacementEnforce::Normal,
          2 => ReplacementEnforce::Post,
          _ => panic!("Invalid enforce value in cached replacement: {}", r.enforce),
        };
        source.replace_with_enforce(r.start, r.end, r.content, r.name, enforce);
      }
      source.boxed()
    }
    CacheableSource::Cached { inner } => CachedSource::new(from_cacheable(*inner)).boxed(),
  }
}
