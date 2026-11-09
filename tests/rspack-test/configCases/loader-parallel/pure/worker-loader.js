const assert = require('node:assert/strict');
const { isMainThread } = require('node:worker_threads');

module.exports = function (content, map, data) {
  assert.equal(isMainThread, false);
  assert.equal(this._compilation.outputOptions.hashFunction, 'sha256');
  assert.equal(data.suffix, 'pure');
  this.callback(null, content, map, { suffix: `${data.suffix}:worker` });
};
