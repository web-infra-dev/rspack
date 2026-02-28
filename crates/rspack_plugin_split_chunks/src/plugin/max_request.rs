use std::borrow::Cow;

use rspack_collections::{DatabaseItem, UkeySet};
use rspack_core::{ChunkUkey, Compilation};

use crate::{CacheGroup, SplitChunksPlugin};

impl SplitChunksPlugin {
  /// Affected by `splitChunks.maxInitialRequests`/`splitChunks.cacheGroups.{cacheGroup}.maxInitialRequests`
  /// Affected by `splitChunks.maxAsyncRequests`/`splitChunks.cacheGroups.{cacheGroup}.maxAsyncRequests`
  // #[tracing::instrument(skip_all)]
  pub(crate) fn ensure_max_request_fit(
    &self,
    compilation: &Compilation,
    cache_group: &CacheGroup,
    used_chunks: &mut Cow<UkeySet<ChunkUkey>>,
  ) {
    let chunk_db = &compilation.build_chunk_graph_artifact.chunk_by_ukey;
    let chunk_group_db = &compilation.build_chunk_graph_artifact.chunk_group_by_ukey;
    let invalided_chunks = used_chunks
      .iter()
      .map(|c| chunk_db.expect_get(c))
      .filter_map(|chunk| {
        let allowed_max_request = if chunk.is_only_initial(chunk_group_db) {
          cache_group.max_initial_requests
        } else if chunk.can_be_initial(chunk_group_db) {
          f64::max(
            cache_group.max_initial_requests,
            cache_group.max_async_requests,
          )
        } else {
          cache_group.max_async_requests
        };

        // `Chunk`s in `used_chunks` are all code-splitting chunk.

        // If a code-splitting chunk is not split by `SplitChunksPlugin`, the number of requests for
        // the chunk is 1.

        // If the code-splitting chunks is split by `SplitChunksPlugin`, to load the code-splitting chunk
        // with correct semantics, we need to also load the chunks derive from the code-splitting chunk.

        // Chunks derive from the code-splitting chunk is in the same ChunkGroup with the split chunk.

        // So the number of requests is the length of `ChunkGroup#chunks` which belong to the split code-splitting
        // chunk.

        let actually_requests = chunk
          .groups()
          .iter()
          .map(|g| chunk_group_db.expect_get(g))
          .map(|group| group.chunks.len())
          .reduce(usize::max)
          .map(|requests| requests as u32)
          .unwrap_or_default();

        if actually_requests as f64 >= allowed_max_request {
          Some(chunk.ukey())
        } else {
          None
        }
      })
      .collect::<Vec<_>>();
    invalided_chunks.into_iter().for_each(|c| {
      used_chunks.to_mut().remove(&c);
    })
  }
}
