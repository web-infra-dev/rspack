use std::hash::{Hash, Hasher};

use bytes_str::BytesStr;
use rustc_hash::{FxHashMap, FxHasher};

use crate::{
  CachedSource, ConcatSource, OriginalSource, ReplaceSource, Source, SourceMap, SourceMapSource,
};

const MISSING: u32 = u32::MAX;
const EMPTY: u32 = u32::MAX - 1;

/// Ordinals of the input source contents used by a generated map.
#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct SourceContentIndices(Vec<u32>);

fn visit_leaves<'a>(source: &'a dyn Source, visit: &mut impl FnMut(&'a dyn Source)) {
  let any = source.as_any();
  if let Some(source) = any.downcast_ref::<ConcatSource>() {
    for child in source.children() {
      visit_leaves(child.as_ref(), visit);
    }
  } else if let Some(source) = any.downcast_ref::<ReplaceSource>() {
    visit_leaves(source.inner().as_ref(), visit);
  } else if let Some(source) = any.downcast_ref::<CachedSource>() {
    visit_leaves(source.inner().as_ref(), visit);
  } else {
    visit(source);
  }
}

/// Asset transforms may replace the source graph while preserving the chunk version.
/// A pass-through map is already available by shallow clone, so skip caching it.
pub fn source_map_input_hash(source: &dyn Source) -> Option<u64> {
  if let Some(source) = source.as_any().downcast_ref::<SourceMapSource>()
    && source.inner_source_map().is_none()
  {
    return None;
  }
  let mut hash = FxHasher::default();
  source.update_hash(&mut hash);
  // The general Source hash omits the intermediate name used to compose maps
  // and the debug ID returned by a pass-through SourceMapSource.
  visit_leaves(source, &mut |leaf| {
    if let Some(source) = leaf.as_any().downcast_ref::<SourceMapSource>() {
      if source.inner_source_map().is_some() {
        source.name().hash(&mut hash);
      }
      source.source_map().get_debug_id().hash(&mut hash);
    }
  });
  Some(hash.finish())
}

fn visit_contents<'a>(source: &'a dyn Source, visit: &mut impl FnMut(&'a BytesStr)) {
  visit_leaves(source, &mut |leaf| {
    let any = leaf.as_any();
    if let Some(source) = any.downcast_ref::<OriginalSource>() {
      visit(&source.value);
    } else if let Some(source) = any.downcast_ref::<SourceMapSource>() {
      for map in std::iter::once(source.source_map()).chain(source.inner_source_map()) {
        for content in map.sources_content().iter().flatten() {
          visit(content);
        }
      }
      if let Some(content) = &source.original_source {
        visit(content);
      }
    }
  });
}

impl SourceContentIndices {
  pub fn new(source: &dyn Source, map: &SourceMap) -> Option<Self> {
    let contents = map.sources_content();
    let mut ordinals = FxHashMap::default();
    if contents.iter().flatten().any(|text| !text.is_empty()) {
      let mut index = 0usize;
      visit_contents(source, &mut |text| {
        ordinals.entry((text.as_ptr(), text.len())).or_insert(index);
        index += 1;
      });
    }

    let content_indices = contents
      .iter()
      .map(|content| match content {
        None => Some(MISSING),
        Some(text) if text.is_empty() => Some(EMPTY),
        Some(text) => {
          let index = *ordinals.get(&(text.as_ptr(), text.len()))?;
          u32::try_from(index).ok().filter(|&index| index < EMPTY)
        }
      })
      .collect::<Option<_>>()?;
    Some(Self(content_indices))
  }

  pub fn restore(&self, source: &dyn Source) -> Option<Vec<Option<BytesStr>>> {
    let mut contents = Vec::new();
    if self.0.iter().any(|&index| index < EMPTY) {
      visit_contents(source, &mut |content| contents.push(content));
    }
    self
      .0
      .iter()
      .map(|&index| match index {
        MISSING => Some(None),
        EMPTY => Some(Some(BytesStr::default())),
        index => Some(Some((*contents.get(index as usize)?).clone())),
      })
      .collect()
  }
}
