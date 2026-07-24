module.exports = process.env.RSPACK_INCREMENTAL_WATCH_TEST
  ? [
      /DeterministicModuleIdsPlugin .* For this rebuild incremental\.moduleIds, incremental\.modulesHashes are fallback to non-incremental/,
    ]
  : [];
