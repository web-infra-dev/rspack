// Invalidate on the first run (the build that first adds late.js) while it is
// still running, so that build is coalesced in `Watching._done` (#12904).
// Key the trigger off the compiler, not a module-level flag: the loader module
// is shared via the worker require cache, so a module-level flag would persist
// across the watchpack and native-watcher suites and skip invalidate() on the
// later run, silently neutering the regression.
const TRIGGERED = Symbol("coalesce-triggered");
module.exports = function (source) {
  const compiler = this._compiler;
  if (compiler && !compiler[TRIGGERED]) {
    compiler[TRIGGERED] = true;
    compiler.watching?.invalidate();
  }
  return source;
};
