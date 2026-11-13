const assert = require('node:assert/strict');
const { isMainThread } = require('node:worker_threads');

module.exports = function (content, map, data) {
  assert.equal(isMainThread, true);
  assert.equal(this.fromLoaderHook, true);
  const buildInfo = this._module.buildInfo;
  if (this.getOptions().seed) {
    buildInfo[Symbol.for('rspack.buildInfo.commitCustomFields')]();
    this.callback(null, content, map, {
      get probe() {
        buildInfo.additionalDataSerialized = true;
        return 'main';
      },
    });
    return;
  }
  assert.equal(buildInfo.additionalDataSerialized, undefined);
  assert.equal(buildInfo.strictMetadata, undefined);
  assert.equal(buildInfo.mainOnly, 'main');
  return `${content}\nmodule.exports += ':${data.suffix}:${this.getOptions().callback()}';`;
};

module.exports.pitch = function () {
  this._module.buildInfo.mainOnly = 'main';
};
