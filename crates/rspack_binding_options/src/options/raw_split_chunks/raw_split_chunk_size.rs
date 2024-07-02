use std::collections::HashMap;

use derivative::Derivative;
use napi_derive::napi;
use rspack_plugin_split_chunks::SplitChunkSizes;

#[derive(Derivative)]
#[napi(object, object_to_js = false)]
#[derivative(Debug)]
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
