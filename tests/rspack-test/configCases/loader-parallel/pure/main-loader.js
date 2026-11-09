const assert = require('node:assert/strict');
const { isMainThread } = require('node:worker_threads');

module.exports = function (content, _map, data) {
  assert.equal(isMainThread, true);
  assert.equal(this.fromLoaderHook, true);
  return `${content}\nmodule.exports += ':${data.suffix}:${this.getOptions().callback()}';`;
};
