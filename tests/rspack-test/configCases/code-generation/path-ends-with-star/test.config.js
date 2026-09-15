// Compile from a per-suite copy so the runtime-created `star*` dir is private
// to this run and the parallel Config.* / RuntimeModeConfig.* suites don't race.
module.exports = {
  isolateSource: true,
};
