use derivative::Derivative;
use napi_derive::napi;
use rspack_plugin_split_chunks::SplitChunkSizes;

#[derive(Derivative)]
#[napi(object, object_to_js = false)]
#[derivative(Debug)]
pub struct RawSplitChunkSizes {
  pub sizes: serde_json::Map<String, serde_json::Value>,
}

impl From<RawSplitChunkSizes> for SplitChunkSizes {
  fn from(sizes: RawSplitChunkSizes) -> Self {
    let mut map = SplitChunkSizes::default();
    for (k, v) in sizes.sizes {
      map.insert((*k).into(), v.as_f64().unwrap_or_default());
    }
    map
  }
}
