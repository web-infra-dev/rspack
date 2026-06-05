// Invalidate on the first run (the build that first adds late.js) while it is
// still running, so that build is coalesced in `Watching._done` (#12904).
let coalesced = false;
module.exports = function (source) {
  if (!coalesced) {
    coalesced = true;
    this._compiler?.watching?.invalidate();
  }
  return source;
};
