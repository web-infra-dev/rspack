use rspack_cacheable::{
  cacheable,
  with::{AsCacheable, AsOption, AsPreset, AsTuple2, AsVec},
};
use rspack_sources::{BoxSource, ConcatSource, SourceExt};

use crate::{AssetInfo, CompilationAsset};

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
#[derive(Debug, Clone)]
pub struct CachedSourceMapDevToolPluginEntry {
  #[cacheable(with=AsVec<AsPreset>)]
  asset_append: Vec<BoxSource>,
  #[cacheable(with=AsOption<AsTuple2<AsCacheable, AsPreset>>)]
  source_map: Option<(String, BoxSource)>,
}

impl CachedSourceMapDevToolPluginEntry {
  pub fn from_assets(
    asset_append: &[BoxSource],
    source_map: Option<(&str, &CompilationAsset)>,
  ) -> Option<Self> {
    let source_map = match source_map {
      Some((filename, asset)) => Some((filename.to_string(), asset.get_source()?.clone())),
      None => None,
    };

    Some(Self {
      asset_append: asset_append.to_vec(),
      source_map,
    })
  }

  #[allow(clippy::type_complexity)]
  pub fn restore(
    &self,
    asset: &CompilationAsset,
  ) -> Option<(
    CompilationAsset,
    Option<(String, CompilationAsset)>,
    Vec<BoxSource>,
  )> {
    let source = asset.get_source()?.clone();
    let source = if self.asset_append.is_empty() {
      source
    } else {
      let mut children = Vec::with_capacity(self.asset_append.len() + 1);
      children.push(source);
      children.extend(self.asset_append.iter().cloned());
      ConcatSource::new(children).boxed()
    };

    let source_asset = CompilationAsset::new(Some(source), (*asset.info).clone());
    let source_map = self.source_map.as_ref().map(|(filename, source)| {
      let mut source_map_asset_info = AssetInfo::default().with_development(Some(true));
      source_map_asset_info.version = asset.info.version.clone();
      (
        filename.clone(),
        CompilationAsset::new(Some(source.clone()), source_map_asset_info),
      )
    });

    Some((source_asset, source_map, self.asset_append.clone()))
  }

  pub(crate) fn asset_append(&self) -> &[BoxSource] {
    &self.asset_append
  }

  pub(crate) fn source_map(&self) -> Option<(&str, &BoxSource)> {
    self
      .source_map
      .as_ref()
      .map(|(filename, source)| (filename.as_str(), source))
  }

  pub(crate) fn from_parts(
    asset_append: Vec<BoxSource>,
    source_map: Option<(String, BoxSource)>,
  ) -> Self {
    Self {
      asset_append,
      source_map,
    }
  }
}
