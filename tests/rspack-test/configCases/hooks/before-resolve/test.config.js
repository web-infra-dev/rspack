// Compile from a per-suite copy so the resolver-generated `.temp/` modules is private to this run and the parallel
// Config.* / RuntimeModeConfig.* suites don't race on the shared source dir.
module.exports = {
  isolateSource: true,
};
