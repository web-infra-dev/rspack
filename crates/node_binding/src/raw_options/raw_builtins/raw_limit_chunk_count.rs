use napi_derive::napi;
use rspack_plugin_limit_chunk_count::LimitChunkCountPluginOptions;

#[derive(Debug, Clone)]
#[napi(object)]
pub struct RawLimitChunkCountPluginOptions {
  // Constant overhead for a chunk.
  pub chunk_overhead: Option<f64>,
  //  Multiplicator for initial chunks.
  pub entry_chunk_multiplicator: Option<f64>,
  // Limit the maximum number of chunks using a value greater greater than or equal to 1.
  pub max_chunks: f64,
}

impl From<RawLimitChunkCountPluginOptions> for LimitChunkCountPluginOptions {
  fn from(value: RawLimitChunkCountPluginOptions) -> Self {
    Self {
      chunk_overhead: value.chunk_overhead,
      entry_chunk_multiplicator: value.entry_chunk_multiplicator,
      max_chunks: value.max_chunks as usize,
    }
  }
}
