// Compile from a per-suite copy so the dts files emitted into `dist/types`
// (resolved against the case context) are private to this run and the
// parallel Config.* / RuntimeModeConfig.* suites don't truncate each other.
module.exports = {
  isolateSource: true,
};
