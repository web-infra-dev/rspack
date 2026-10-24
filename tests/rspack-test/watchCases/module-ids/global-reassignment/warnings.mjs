export default process.env.RSPACK_INCREMENTAL_WATCH_TEST
  ? [
      /DeterministicModuleIdsPlugin .* For this rebuild incremental\.moduleIds are fallback to non-incremental/,
    ]
  : [];
