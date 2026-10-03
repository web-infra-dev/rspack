//! Cacheable source types for serialization/deserialization.
#![allow(missing_docs)]

use crate::{
  BoxSource, CachedSource, ConcatSource, OriginalSource, RawBufferSource, RawStringSource,
  ReplaceSource, ReplacementEnforce, Source, SourceExt, SourceMap, SourceMapSource,
  SourceMapSourceOptions, SourceSlice,
};

/// Serializable representation of a [`Replacement`](crate::Replacement).
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(serialize_bounds(
  __S: rkyv::ser::Writer + rkyv::ser::Allocator + rkyv::rancor::Fallible<Error: rkyv::rancor::Source>
))]
#[rkyv(deserialize_bounds(
  __D: rkyv::rancor::Fallible<Error: rkyv::rancor::Source>
))]
#[rkyv(bytecheck(bounds(
  __C: rkyv::validation::ArchiveContext + rkyv::rancor::Fallible<Error: rkyv::rancor::Source>
)))]
pub struct CacheableReplacement {
  /// Start offset.
  pub start: u32,
  /// End offset.
  pub end: u32,
  /// Replacement content.
  #[rkyv(omit_bounds)]
  pub content: Box<CacheableSource>,
  /// Replacement name.
  pub name: Option<String>,
  /// Enforce order: 0 = Pre, 1 = Normal, 2 = Post.
  pub enforce: u8,
  /// Original insertion order, retained when the sorted edits are restored.
  pub insertion_order: u32,
}

/// Serializable representation of a [`BoxSource`].
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(serialize_bounds(
  __S: rkyv::ser::Writer + rkyv::ser::Allocator + rkyv::rancor::Fallible<Error: rkyv::rancor::Source>
))]
#[rkyv(deserialize_bounds(
  __D: rkyv::rancor::Fallible<Error: rkyv::rancor::Source>
))]
#[rkyv(bytecheck(bounds(
  __C: rkyv::validation::ArchiveContext + rkyv::rancor::Fallible<Error: rkyv::rancor::Source>
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
    #[rkyv(omit_bounds)]
    replacements: Vec<CacheableReplacement>,
  },
  /// [`SourceSlice`]
  Slice {
    #[rkyv(omit_bounds)]
    inner: Box<CacheableSource>,
    start: u32,
    end: u32,
  },
  /// [`CachedSource`]
  Cached {
    /// The inner source.
    #[rkyv(omit_bounds)]
    inner: Box<CacheableSource>,
  },
}

/// Convert a [`Source`] trait object into a serializable [`CacheableSource`].
pub fn to_cacheable(source: &dyn Source) -> CacheableSource {
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
        insertion_order: r.insertion_order(),
        start: r.start(),
        end: r.end(),
        content: Box::new(to_cacheable(r.content().as_ref())),
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

  if let Some(s) = source.as_any().downcast_ref::<SourceSlice>() {
    return CacheableSource::Slice {
      inner: Box::new(to_cacheable(s.inner().as_ref())),
      start: s.range().start as u32,
      end: s.range().end as u32,
    };
  }

  panic!(
    "Unexpected source type in persistent cache serialization. All BoxSource instances should be one of the known rspack_sources types."
  )
}

/// Convert a [`CacheableSource`] back into a [`BoxSource`].
pub fn from_cacheable(cacheable: CacheableSource) -> BoxSource {
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
      mut replacements,
    } => {
      let inner = from_cacheable(*inner);
      let mut source = ReplaceSource::new(inner);
      replacements.sort_unstable_by_key(|r| r.insertion_order);
      for r in replacements {
        let enforce = match r.enforce {
          0 => ReplacementEnforce::Pre,
          1 => ReplacementEnforce::Normal,
          2 => ReplacementEnforce::Post,
          _ => {
            panic!("Invalid enforce value in cached replacement: {}", r.enforce)
          }
        };
        source.replace_source_with_enforce(
          r.start,
          r.end,
          from_cacheable(*r.content),
          r.name,
          enforce,
        );
      }
      source.boxed()
    }
    CacheableSource::Slice { inner, start, end } => {
      SourceSlice::new(from_cacheable(*inner), start as usize..end as usize).boxed()
    }
    CacheableSource::Cached { inner } => CachedSource::new(from_cacheable(*inner)).boxed(),
  }
}
