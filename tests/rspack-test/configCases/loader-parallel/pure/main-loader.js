const assert = require('node:assert/strict');
const { isMainThread } = require('node:worker_threads');

module.exports = function (content, _map, data) {
  assert.equal(isMainThread, true);
  assert.equal(this.fromLoaderHook, true);
  assert.deepEqual(this._module.buildInfo.pureMetadata, { phase: 'normal' });
  assert.equal(this._module.buildInfo.mainOnly(), 'main');
  return `${content}\nmodule.exports += ':${data.suffix}:${this.getOptions().callback()}';`;
};

module.exports.pitch = function () {
  this._module.buildInfo.mainOnly = () => 'main';
};
