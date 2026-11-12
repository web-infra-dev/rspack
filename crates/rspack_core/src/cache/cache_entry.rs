use cow_utils::CowUtils;
use rspack_cacheable::{cacheable, with::AsPreset};
use rspack_sources::{
  BoxSource, DevToolCodeSource, DevToolMapSource, RawStringSource, SourceExt, SourceMap,
  cacheable::CompactSourceMap, is_buffer_source,
};
use rspack_util::base64;

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
#[derive(Debug)]
pub struct CachedSourceMapDevToolPluginEntry {
  pub map: CompactSourceMap,
  pub filename: Option<String>,
  /// Fully rendered external suffix, or the inline template containing `[url]`.
  pub comment: String,
}

impl CachedSourceMapDevToolPluginEntry {
  pub fn render_inline_comment(template: &str, map: &SourceMap) -> String {
    let base64 = base64::encode_to_string(map.to_json().as_bytes());
    template
      .cow_replace(
        "[url]",
        &format!("data:application/json;charset=utf-8;base64,{base64}"),
      )
      .into_owned()
  }

  pub fn restore(
    &self,
    asset: &CompilationAsset,
    detach_source: bool,
  ) -> Option<(CompilationAsset, Option<(String, CompilationAsset)>)> {
    let source = asset.get_source()?;
    // A cache hit skips map collection. Never apply cached DevTool output to a buffer.
    if is_buffer_source(source.as_ref()) {
      return None;
    }
    let map = self.map.restore(source.as_ref())?;
    let (suffix, source_map) = if let Some(filename) = &self.filename {
      let mut info = AssetInfo::default().with_development(Some(true));
      info.version = asset.info.version.clone();
      let map_asset = CompilationAsset::new(Some(DevToolMapSource::new(map).boxed()), info);
      (self.comment.clone(), Some((filename.clone(), map_asset)))
    } else {
      (Self::render_inline_comment(&self.comment, &map), None)
    };
    let code = if detach_source {
      RawStringSource::from(source.source().into_string_lossy().into_owned()).boxed()
    } else {
      source.clone()
    };
    let source = DevToolCodeSource::new(code, RawStringSource::from(suffix).boxed()).boxed();
    Some((
      CompilationAsset::new(Some(source), (*asset.info).clone()),
      source_map,
    ))
  }
}
