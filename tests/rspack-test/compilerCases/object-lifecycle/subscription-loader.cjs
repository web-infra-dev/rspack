let previousModule;

module.exports = function () {
  const { name, parallel } = this.getOptions();
  require("node:assert/strict").equal(require("node:worker_threads").isMainThread, !parallel);
  require("node:assert/strict").equal(this._compilation.compiler.options.name, name);
  if (!parallel) require("node:assert/strict").equal(this._compiler.options.name, name);
  if (parallel) {
    if (previousModule) {
      require("node:assert/strict").throws(
        () => previousModule.nameForCondition(),
        /loader task has completed/
      );
    }
    require("node:assert/strict").equal(this._module.nameForCondition(), this.resourcePath);
    previousModule = this._module;
  }
  const callback = this.async();
  // Overlap callbacks from different compilers to exercise compiler routing.
  setTimeout(() => callback(null, `module.exports = ${JSON.stringify(`subscription:${name}`)};`), 5);
};
