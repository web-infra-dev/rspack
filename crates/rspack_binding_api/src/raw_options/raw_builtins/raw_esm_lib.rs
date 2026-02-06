use rspack_plugin_split_chunks::CacheGroup;

use crate::raw_options::RawSplitChunksOptions;

#[napi(object, object_to_js = false)]
pub struct RawEsmLibraryPlugin<'a> {
  pub preserve_modules: Option<String>,
  pub split_chunks: Option<RawSplitChunksOptions<'a>>,
}

impl<'a> From<RawSplitChunksOptions<'a>> for Vec<CacheGroup> {
  fn from(value: RawSplitChunksOptions<'a>) -> Self {
    let mut groups = rspack_plugin_split_chunks::PluginOptions::from(value).cache_groups;

    groups.sort_by(|a, b| b.priority.total_cmp(&a.priority));

    groups
  }
}
