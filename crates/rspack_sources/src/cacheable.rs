//! Cacheable source types for serialization/deserialization.
#![allow(missing_docs)]

use bytes_str::BytesStr;
use rkyv::{
  Place,
  rancor::{Fallible, Source as RkyvSource},
  ser::Writer,
  string::{ArchivedString, StringResolver},
  with::{ArchiveWith, DeserializeWith, Map, SerializeWith},
};
use triomphe::ThinArc;

use crate::{
  BoxSource, CachedSource, ConcatSource, DevToolCodeSource, DevToolMapSource, OriginalSource,
  RawBufferSource, RawStringSource, ReplaceSource, ReplacementEnforce, Source, SourceExt,
  SourceMap, SourceMapSource, SourceMapSourceOptions, source::shared_slice,
};

/// Source map fields stored without an intermediate JSON representation.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct CacheableSourceMap {
  version: u8,
  #[rkyv(with = Map<AsStr>)]
  file: Option<BytesStr>,
  #[rkyv(with = Map<AsStr>)]
  sources: Vec<BytesStr>,
  #[rkyv(with = Map<Map<AsStr>>)]
  sources_content: Vec<Option<BytesStr>>,
  #[rkyv(with = Map<AsStr>)]
  names: Vec<BytesStr>,
  #[rkyv(with = AsStr)]
  mappings: BytesStr,
  #[rkyv(with = Map<AsStr>)]
  source_root: Option<BytesStr>,
  #[rkyv(with = Map<AsStr>)]
  debug_id: Option<BytesStr>,
  ignore_list: Option<Vec<u32>>,
}

// Archive text as strings so cache validation checks UTF-8 before restoring BytesStr.
struct AsStr;

impl ArchiveWith<BytesStr> for AsStr {
  type Archived = ArchivedString;
  type Resolver = StringResolver;

  fn resolve_with(field: &BytesStr, resolver: Self::Resolver, out: Place<Self::Archived>) {
    ArchivedString::resolve_from_str(field.as_str(), resolver, out);
  }
}

impl<S> SerializeWith<BytesStr, S> for AsStr
where
  S: Writer + Fallible + ?Sized,
  S::Error: RkyvSource,
{
  fn serialize_with(field: &BytesStr, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
    ArchivedString::serialize_from_str(field.as_str(), serializer)
  }
}

impl<D: Fallible + ?Sized> DeserializeWith<ArchivedString, BytesStr, D> for AsStr {
  fn deserialize_with(field: &ArchivedString, _: &mut D) -> Result<BytesStr, D::Error> {
    Ok(BytesStr::from_str_slice(field.as_str()))
  }
}

impl From<&SourceMap> for CacheableSourceMap {
  fn from(map: &SourceMap) -> Self {
    Self {
      version: map.version,
      file: map.file.clone(),
      sources: map.sources().to_vec(),
      sources_content: map.sources_content().to_vec(),
      names: map.names().to_vec(),
      mappings: map.mappings.clone(),
      source_root: map.source_root.clone(),
      debug_id: map.debug_id.clone(),
      ignore_list: map.ignore_list().map(<[u32]>::to_vec),
    }
  }
}

impl From<CacheableSourceMap> for SourceMap {
  fn from(map: CacheableSourceMap) -> Self {
    SourceMap {
      version: map.version,
      file: map.file,
      sources: shared_slice(map.sources.into_iter()),
      sources_content: shared_slice(map.sources_content.into_iter()),
      names: shared_slice(map.names.into_iter()),
      mappings: map.mappings,
      source_root: map.source_root,
      debug_id: map.debug_id,
      ignore_list: map
        .ignore_list
        .map(|items| ThinArc::from_header_and_iter((), items.into_iter())),
    }
  }
}

/// Serializable representation of a [`Replacement`](crate::Replacement).
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct CacheableReplacement {
  /// Start offset.
  pub start: u32,
  /// End offset.
  pub end: u32,
  /// Replacement content.
  pub content: String,
  /// Replacement name.
  pub name: Option<String>,
  /// Enforce order: 0 = Pre, 1 = Normal, 2 = Post.
  pub enforce: u8,
}

/// Serializable representation of a [`BoxSource`].
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(serialize_bounds(
  __S: rkyv::ser::Writer + rkyv::ser::Allocator + rkyv::rancor::Fallible
))]
#[rkyv(deserialize_bounds(
  __D: rkyv::rancor::Fallible<Error: rkyv::rancor::Source>
))]
#[rkyv(bytecheck(bounds(
  __C: rkyv::validation::ArchiveContext + rkyv::rancor::Fallible
)))]
pub enum CacheableSource {
  /// [`RawBufferSource`]
  RawBuffer {
    /// The raw buffer.
    buffer: Vec<u8>,
  },
  /// [`RawStringSource`]
  RawString {
    /// The string value.
    value: String,
  },
  /// [`OriginalSource`]
  Original {
    /// The source value.
    value: String,
    /// The source name.
    name: String,
  },
  /// [`SourceMapSource`]
  SourceMap {
    /// The source value.
    value: String,
    /// The source name.
    name: String,
    /// The source map JSON.
    source_map: String,
    /// The optional original source.
    original_source: Option<String>,
    /// The optional inner source map JSON.
    inner_source_map: Option<String>,
    /// Whether to remove the original source.
    remove_original_source: bool,
  },
  /// [`ConcatSource`]
  Concat {
    /// Child sources.
    #[rkyv(omit_bounds)]
    children: Vec<CacheableSource>,
  },
  /// [`ReplaceSource`]
  Replace {
    /// The inner source.
    #[rkyv(omit_bounds)]
    inner: Box<CacheableSource>,
    /// The replacements.
    replacements: Vec<CacheableReplacement>,
  },
  /// [`CachedSource`]
  Cached {
    /// The inner source.
    #[rkyv(omit_bounds)]
    inner: Box<CacheableSource>,
  },
  /// [`DevToolMapSource`], preserving every field, including omitted empty arrays.
  DevToolMap {
    /// Keep map metadata from enlarging every cached source node.
    map: Box<CacheableSourceMap>,
  },
  /// [`DevToolCodeSource`], preserving the shared code and appended comment.
  DevToolCode {
    #[rkyv(omit_bounds)]
    inner: Box<CacheableSource>,
    suffix: String,
  },
}

/// Convert a [`Source`] trait object into a serializable [`CacheableSource`].
pub fn to_cacheable(source: &dyn Source) -> CacheableSource {
  if let Some(s) = source.as_any().downcast_ref::<DevToolMapSource>() {
    return CacheableSource::DevToolMap {
      map: Box::new(s.original_map().into()),
    };
  }
  if let Some(s) = source.as_any().downcast_ref::<DevToolCodeSource>() {
    return CacheableSource::DevToolCode {
      inner: Box::new(to_cacheable(s.inner().as_ref())),
      suffix: s.suffix().source().into_string_lossy().into_owned(),
    };
  }
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

/// Convert a [`CacheableSource`] back into a [`BoxSource`].
pub fn from_cacheable(cacheable: CacheableSource) -> BoxSource {
  match cacheable {
    CacheableSource::DevToolMap { map } => DevToolMapSource::new((*map).into()).boxed(),
    CacheableSource::DevToolCode { inner, suffix } => DevToolCodeSource::new(
      from_cacheable(*inner),
      RawStringSource::from(suffix).boxed(),
    )
    .boxed(),
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
      let source_map = SourceMap::from_json(source_map).expect("invalid source map JSON");
      let inner_source_map = inner_source_map.and_then(|json| SourceMap::from_json(json).ok());
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
          _ => {
            panic!("Invalid enforce value in cached replacement: {}", r.enforce)
          }
        };
        source.replace_with_enforce(r.start, r.end, r.content, r.name, enforce);
      }
      source.boxed()
    }
    CacheableSource::Cached { inner } => CachedSource::new(from_cacheable(*inner)).boxed(),
  }
}
