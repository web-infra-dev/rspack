const assert = require('node:assert/strict');
const { isMainThread } = require('node:worker_threads');

module.exports = function (content) {
  assert.equal(isMainThread, false);
  assert.equal(this.fromMainHook, undefined);
  assert.equal(this.data.pitched, true);
  return content;
};
module.exports.pitch = function (_remaining, _previous, data) {
  assert.equal(isMainThread, false);
  assert.equal(this.fromMainHook, undefined);
  data.pitched = true;
};
