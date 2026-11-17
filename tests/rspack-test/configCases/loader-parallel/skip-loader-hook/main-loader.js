const assert = require('node:assert/strict');
const { isMainThread } = require('node:worker_threads');

module.exports = function (content) {
  assert.equal(isMainThread, true);
  assert.equal(this.fromMainHook, 'main');
  return `${content}\nmodule.exports += ':main';`;
};
