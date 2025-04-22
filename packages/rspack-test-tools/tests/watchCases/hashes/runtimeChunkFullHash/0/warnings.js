module.exports = process.env.RSPACK_INCREMENTAL_WATCH_TEST
  ? [[/Chunks that dependent on full hash requires calculating the hashes of all chunks, which is a global effect/, /`incremental.chunksHashes` has been overridden to false/]]
  : [];
