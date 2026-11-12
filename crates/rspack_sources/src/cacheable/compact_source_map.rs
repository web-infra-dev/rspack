use bytes_str::BytesStr;
use rustc_hash::FxHashMap;

use super::CacheableSourceMap;
use crate::{
  CachedSource, ConcatSource, OriginalSource, ReplaceSource, Source, SourceMap, SourceMapSource,
};

const MISSING: u32 = u32::MAX;
const EMPTY: u32 = u32::MAX - 1;

/// Source map metadata plus references into the input asset's source graph.
#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct CompactSourceMap {
  metadata: CacheableSourceMap,
  content_indices: Vec<u32>,
}

// Cache writers and readers must visit the same source graph in the same order.
fn visit_contents<'a>(source: &'a dyn Source, visit: &mut impl FnMut(&'a BytesStr)) {
  let source = source.as_any();
  if let Some(source) = source.downcast_ref::<OriginalSource>() {
    visit(&source.value);
  } else if let Some(source) = source.downcast_ref::<SourceMapSource>() {
    for map in std::iter::once(source.source_map()).chain(source.inner_source_map()) {
      for content in map.sources_content().iter().flatten() {
        visit(content);
      }
    }
    if let Some(content) = &source.original_source {
      visit(content);
    }
  } else if let Some(source) = source.downcast_ref::<ConcatSource>() {
    for child in source.children() {
      visit_contents(child.as_ref(), visit);
    }
  } else if let Some(source) = source.downcast_ref::<ReplaceSource>() {
    visit_contents(source.inner().as_ref(), visit);
  } else if let Some(source) = source.downcast_ref::<CachedSource>() {
    visit_contents(source.inner().as_ref(), visit);
  }
}

impl CompactSourceMap {
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
    let mut metadata = map.clone();
    metadata.set_sources_content(Vec::new());
    Some(Self {
      metadata: (&metadata).into(),
      content_indices,
    })
  }

  pub fn restore(&self, source: &dyn Source) -> Option<SourceMap> {
    let mut contents = Vec::new();
    if self.content_indices.iter().any(|&index| index < EMPTY) {
      visit_contents(source, &mut |content| contents.push(content));
    }
    let restored = self
      .content_indices
      .iter()
      .map(|&index| match index {
        MISSING => Some(None),
        EMPTY => Some(Some(BytesStr::default())),
        index => Some(Some((*contents.get(index as usize)?).clone())),
      })
      .collect::<Option<_>>()?;
    let mut map = SourceMap::from(self.metadata.clone());
    map.set_sources_content(restored);
    Some(map)
  }
}
