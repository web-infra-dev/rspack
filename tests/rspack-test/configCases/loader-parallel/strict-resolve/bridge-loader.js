const assert = require('node:assert/strict');
const { isMainThread } = require('node:worker_threads');
module.exports = function (source, map, data) {
  assert.equal(this.fromHook, undefined);
  assert.equal(isMainThread, false);
  assert.equal(this.getOptions().callback(), 'main API');
  assert.equal(typeof this._compiler.context, 'string');
  assert.equal(typeof this._compilation.getLogger, 'function');
  assert.equal(data, undefined);
  this.callback(null, source, map);
};
