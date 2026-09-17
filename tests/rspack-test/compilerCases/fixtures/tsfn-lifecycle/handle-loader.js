const assert = require("node:assert/strict");

module.exports = function (source, map, additionalData) {
  // Repeated compiler.run() calls must exercise the loader bridge again.
  this.cacheable(false);
  const { role, fail } = this.getOptions();
  assert.equal(this.handleState.marker, "loader-hook");
  assert.equal(this.handleContext(), this);
  if (role === "producer") {
    this.handleState.produced++;
    if (fail) throw new Error("intentional handle test failure");
    this.callback(null, source, map, { marker: "additional-data" });
  } else {
    assert.deepEqual(additionalData, { marker: "additional-data" });
    this.handleState.consumed++;
    this.callback(null, source, map);
  }
};
