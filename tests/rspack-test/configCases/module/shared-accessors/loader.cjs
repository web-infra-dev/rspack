module.exports = function (source) {
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
