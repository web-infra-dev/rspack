use std::collections::HashMap;

use napi_derive::napi;
use rspack_plugin_split_chunks::SplitChunkSizes;

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawSplitChunkSizes {
  pub sizes: HashMap<String, f64>,
}

impl From<RawSplitChunkSizes> for SplitChunkSizes {
  fn from(sizes: RawSplitChunkSizes) -> Self {
    let mut split_chunk_sizes = SplitChunkSizes::default();
    for (chunk_name, size) in sizes.sizes {
      split_chunk_sizes.insert((*chunk_name).into(), size);
    }
    split_chunk_sizes
  }
}
