export default process.env.RSPACK_INCREMENTAL_WATCH_TEST
  ? [/FlagDependencyUsagePlugin \(optimization\.usedExports = true\).*incremental\.modulesHashes are fallback to non-incremental/]
  : [];
