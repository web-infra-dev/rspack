const assert = require('node:assert/strict');

module.exports = function (source) {
  assert.equal(this.hot, true);
  this._compiler.loaderHookEvents.push('normal:' + this.getOptions().name);
  return source;
};

module.exports.pitch = function () {
  assert.equal(this.hot, true);
  this._compiler.loaderHookEvents.push('pitch:' + this.getOptions().name);
};
