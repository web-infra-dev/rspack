const assert = require('node:assert/strict');
let setMatchResource;

module.exports = async function (source) {
  // Exercise the setters on every build: cache reuse refreshes factoryMeta.
  this.cacheable(false);
  const descriptor = Object.getOwnPropertyDescriptor(this._module, 'matchResource');
  setMatchResource ||= descriptor.set;
  assert.equal(descriptor.set, setMatchResource);
  assert.equal(this._module.matchResource, undefined);
  await new Promise((resolve) => setImmediate(resolve));
  setMatchResource.call(this._module, `${this.resourcePath}?shared#test`);
  assert.equal(this._module.matchResource, `${this.resourcePath}?shared#test`);
  // Assigning undefined has always been a no-op, not a reset.
  setMatchResource.call(this._module, undefined);
  assert.equal(this._module.matchResource, `${this.resourcePath}?shared#test`);
  this.addDependency(this.resourcePath);
  this.addContextDependency(this.context);
  this.addMissingDependency(`${this.resourcePath}.missing`);
  this.addBuildDependency(__filename);
  this._module.factoryMeta = { sideEffectFree: false };
  this._module.buildInfo = { resource: this.resourcePath };
  this.emitFile(
    `${this.resourcePath.endsWith('index.js') ? 'index' : 'value'}.txt`,
    'asset',
  );
  return source;
};
