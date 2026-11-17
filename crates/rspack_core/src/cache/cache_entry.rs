use rspack_cacheable::{
  cacheable,
  with::{As, AsPreset},
};
use rspack_sources::{
  BoxSource, Source, SourceMap,
  cacheable::{CacheableSourceMap, SourceContentIndices},
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct CachedMinimizeEntry {
  #[cacheable(with=AsPreset)]
  pub source: BoxSource,
  pub extracted_comments: Option<CachedExtractedComments>,
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct CachedExtractedComments {
  #[cacheable(with=AsPreset)]
  pub source: BoxSource,
  pub comments_file_name: String,
}

#[cacheable]
#[derive(Debug)]
pub struct CachedSourceMapDevToolPluginEntry {
  #[cacheable(with=As<CacheableSourceMap>)]
  map: SourceMap,
  content_indices: Option<SourceContentIndices>,
}

impl CachedSourceMapDevToolPluginEntry {
  pub fn new(
    source: &dyn Source,
    map: &SourceMap,
    persistent: bool,
    no_sources: bool,
  ) -> Option<Self> {
    let mut map = map.clone();
    let content_indices = if persistent && !no_sources && !map.sources_content().is_empty() {
      Some(SourceContentIndices::new(source, &map)?)
    } else {
      None
    };
    if persistent || no_sources {
      map.set_sources_content(Vec::new());
      // Metadata sliced from a parsed JSON buffer can otherwise retain all the omitted text.
      map.shrink_to_fit();
    }
    Some(Self {
      map,
      content_indices,
    })
  }

  pub fn restore(&self, source: &dyn Source) -> Option<SourceMap> {
    let mut map = self.map.clone();
    if let Some(indices) = &self.content_indices {
      map.set_sources_content(indices.restore(source)?);
    }
    Some(map)
  }
}
