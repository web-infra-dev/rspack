const assert = require('node:assert/strict');

module.exports = function (source) {
  return source;
};

module.exports.pitch = function () {
  const { runtime, hookWorker, mode } = this.getOptions();
  assert.deepEqual(this[Symbol.for('css-extract-rspack-plugin')], { runtime });
  assert.equal(this.hookWorker, hookWorker);
  assert.equal(this.readHookWorker(), hookWorker);
  assert.deepEqual(
    this.hookOrder,
    mode === 'mixed'
      ? ['worker', 'main']
      : mode === 'interceptor'
        ? ['interceptor', 'worker']
        : ['worker'],
  );
};
