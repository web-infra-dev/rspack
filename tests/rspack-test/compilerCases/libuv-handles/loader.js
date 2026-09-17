const assert = require("node:assert/strict");

module.exports = function (source, map, additionalData) {
  // Repeated compiler.run() calls must exercise the loader bridge again.
  this.cacheable(false);
  const { role, fail } = this.getOptions();
  assert.equal(this.loaderTestState.marker, "loader-hook");
  assert.equal(this.getCapturedLoaderContext(), this);
  if (role === "producer") {
    this.loaderTestState.producerCalls++;
    if (fail) throw new Error("intentional handle test failure");
    this.callback(null, source, map, { marker: "additional-data" });
  } else {
    assert.deepEqual(additionalData, { marker: "additional-data" });
    this.loaderTestState.consumerCalls++;
    this.callback(null, source, map);
  }
};
