const assert = require('node:assert/strict');
const { isMainThread } = require('node:worker_threads');
module.exports = function (source, map, data) {
  assert.equal(this.fromHook, true);
  assert.equal(isMainThread, true);
  if (this.getOptions().seed) {
    this._module.buildInfo.mainOnly = 'main';
    this.callback(null, source, map, () => 'main additionalData');
  } else {
    assert.equal(data, undefined);
    this.callback(null, source, map);
  }
};
