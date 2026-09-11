const assert = require('node:assert/strict');
module.exports = function () {
  const { fn, local, parallel } = this.getOptions();
  assert.equal(require('node:worker_threads').isMainThread, !parallel);
  assert.equal(this.prepared(0), 42);
  // getOptions and both prepared functions remain synchronous, including ESM exports.
  return `module.exports = ${fn(0) + local(0)}`;
};
