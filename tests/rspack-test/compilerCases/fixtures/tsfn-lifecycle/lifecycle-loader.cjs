const assert = require('node:assert/strict');

module.exports = function (source) {
  assert.equal(this.fromHook, 'preserved');
  assert.equal(this.hookSelf(), this);
  return source;
};

module.exports.pitch = function () {
  assert.equal(this.fromHook, 'preserved');
  assert.equal(this.hookSelf(), this);
};
