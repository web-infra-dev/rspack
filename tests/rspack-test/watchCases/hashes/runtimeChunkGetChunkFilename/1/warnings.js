module.exports = process.env.RSPACK_INCREMENTAL_WATCH_TEST
  ? [/Chunk content that dependent on full hash is not friendly for incremental, it requires calculating the hashes of all the chunks, which is a global effect. For this rebuild incremental\.chunksHashes are fallback to non-incremental/]
  : [];
