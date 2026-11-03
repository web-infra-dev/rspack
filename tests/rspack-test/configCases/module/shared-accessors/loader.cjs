const assert = require('node:assert/strict');
let setMatchResource;

module.exports = function (source) {
  const descriptor = Object.getOwnPropertyDescriptor(
    this._module,
    'matchResource',
  );
  setMatchResource ||= descriptor.set;
  assert.equal(descriptor.set, setMatchResource);
  assert.equal(this._module.matchResource, undefined);
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
  const previousBuildInfo = this._module.buildInfo;
  const module = this._module;
  const resourcePath = this.resourcePath;
  module.buildInfo = {
    get resource() {
      assert.equal(module.buildInfo, previousBuildInfo);
      return resourcePath;
    },
  };
  assert.notEqual(previousBuildInfo, this._module.buildInfo);
  assert.deepEqual(
    previousBuildInfo.fileDependencies,
    this._module.buildInfo.fileDependencies,
  );
  const info = this._module.buildInfo;
  const filesKey = Symbol.for('rspack.buildInfo.fileDependencies');
  const commitKey = Symbol.for('rspack.buildInfo.commitCustomFields');
  let reads = 0;
  Object.defineProperty(info, 'resource', {
    enumerable: true,
    configurable: true,
    get: () => {
      reads++;
      // Custom getters may re-enter the module cache through native BuildInfo.
      assert.deepEqual(info[filesKey], previousBuildInfo[filesKey]);
      return this.resourcePath;
    },
  });
  info[commitKey]();
  assert.equal(reads, 1);
  Object.defineProperty(info, 'resource', {
    value: this.resourcePath,
    writable: true,
    enumerable: true,
    configurable: true,
  });
  this.emitFile(
    `${this.resourcePath.endsWith('index.js') ? 'index' : 'value'}.txt`,
    'asset',
  );
  return source;
};
