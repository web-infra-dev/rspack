const assert = require('node:assert/strict');
const path = require('node:path');

function checkHookContext(context) {
  assert.equal(context.hookValue, 'from loader hook');
  assert.equal(context.hookContext, context);
  assert.equal(context[Symbol.for('loader-hook-value')].value, 42);
  const dependency = path.resolve(__dirname, 'rspack.config.js');
  context.clearDependencies();
  context.addHookDependency();
  assert(context.getDependencies().includes(dependency));
}

module.exports = function (source) {
  checkHookContext(this);
  assert.equal(this.hot, true);
  this._compiler.loaderHookEvents.push('normal:' + this.getOptions().name);
  return source;
};

module.exports.pitch = function () {
  checkHookContext(this);
  assert.equal(this.hot, true);
  this._compiler.loaderHookEvents.push('pitch:' + this.getOptions().name);
};
