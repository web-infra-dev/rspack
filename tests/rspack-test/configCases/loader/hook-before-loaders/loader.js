const assert = require('node:assert/strict');
const path = require('node:path');

function checkHookContext(context) {
  assert.equal(context.loaders, context.hookLoaders);
  context.loaders.forEach((loader, index) => {
    assert.equal(loader, context.hookLoaderObjects[index]);
  });
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
  assert.equal(this.hot, false);
  const name = this.getOptions().name;
  assert.equal(this.data.name, name);
  assert(this.hookLoaderObjects.every(loader => loader.pitchExecuted));
  assert(this.hookLoaderObjects.slice(this.loaderIndex).every(loader => loader.normalExecuted));
  this._compiler.loaderHookEvents.push('normal:' + name);
  return source;
};

module.exports.pitch = function () {
  checkHookContext(this);
  assert.equal(this.hot, false);
  this.data.name = this.getOptions().name;
  assert(this.hookLoaderObjects.slice(0, this.loaderIndex + 1).every(loader => loader.pitchExecuted));
  this._compiler.loaderHookEvents.push('pitch:' + this.getOptions().name);
};
