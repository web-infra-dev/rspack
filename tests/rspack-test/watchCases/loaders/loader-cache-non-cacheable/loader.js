let loaderRuns = 0;

module.exports = function (source) {
  this.addDependency(`${this.rootContext}/trigger.js`);
  this.cacheable(false);
  loaderRuns++;
  return source.replace('__LOADER_RUNS__', loaderRuns);
};
