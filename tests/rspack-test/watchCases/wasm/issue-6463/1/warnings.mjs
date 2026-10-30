export default process.env.RSPACK_INCREMENTAL_WATCH_TEST
  ? [/Chunk content that dependent on full hash is not friendly for incremental/]
  : [];
