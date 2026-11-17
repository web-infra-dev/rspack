const assert = require('node:assert/strict');
const { isMainThread } = require('node:worker_threads');

module.exports = function (content, map, data) {
  assert.equal(isMainThread, false);
  // main's normal loader hook runs before dispatching an ordinary parallel loader.
  assert.equal(this.getOptions().fromRegularParallelHook, true);
  assert.equal(this._compilation.outputOptions.hashFunction, 'sha256');
  assert.equal(data, undefined);
  assert.equal(this._module.buildInfo.strictMetadata, undefined);
  this.callback(null, content, map, { suffix: 'worker' });
};
