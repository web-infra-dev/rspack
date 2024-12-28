use napi_derive::napi;
use rspack_ids::OccurrenceChunkIdsPluginOptions;

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawOccurrenceChunkIdsPluginOptions {
  pub prioritise_initial: Option<bool>,
}

impl From<RawOccurrenceChunkIdsPluginOptions> for OccurrenceChunkIdsPluginOptions {
  fn from(value: RawOccurrenceChunkIdsPluginOptions) -> Self {
    Self {
      prioritise_initial: value.prioritise_initial.unwrap_or_default(),
    }
  }
}
