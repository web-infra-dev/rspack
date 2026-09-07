module.exports = process.env.RSPACK_INCREMENTAL_WATCH_TEST
  ? [
      /DeterministicChunkIdsPlugin .* For this rebuild incremental\.chunkIds, incremental\.modulesHashes are fallback to non-incremental/,
    ]
  : [];
