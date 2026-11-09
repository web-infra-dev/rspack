const assert = require("node:assert/strict");

module.exports = function () {
  const module = this._module;
  assert.equal(this.parallel, true);
  assert.equal(module.resource, this.resourcePath);
  assert.equal(module.context, this.context);
  assert.equal(module.type, "javascript/auto");
  assert.ok(module.identifier().includes("value.js"));
  assert.ok(module.readableIdentifier().includes("value.js"));
  assert.equal(module.nameForCondition(), this.resourcePath);
  assert.equal(typeof module.size(), "number");
  assert.equal(module.originalSource(), null);
  assert.equal("blocks" in module, false);
  assert.equal("dependencies" in module, false);
  assert.equal("error" in module, false);
  module.factoryMeta = { sideEffectFree: false };
  module.matchResource = "worker-resource.js";
  module.buildInfo.nativeWorker = true;
  assert.equal(module.buildInfo.fromBuildModule(), "main buildModule");
  this.emitFile("worker-module.txt", "from native worker", undefined, {
    immutable: true,
    sourceFilename: "value.js"
  });
  assert.ok(Object.keys(module.buildInfo.assets).includes("worker-module.txt"));
  return 'module.exports = "native module";';
};
