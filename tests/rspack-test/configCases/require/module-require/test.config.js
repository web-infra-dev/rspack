// Compile from a per-suite copy so the runtime-written `foo/bar/a.js` and `posix-backslash.generated.js` is private to this run and the parallel
// Config.* / RuntimeModeConfig.* suites don't race on the shared source dir.
module.exports = {
  isolateSource: true,
};
