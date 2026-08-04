let loaderRuns = 0;

module.exports = function (source) {
  this.addDependency(`${this.rootContext}/trigger.js`);
  this.emitFile('loader-cache-side-effect.txt', String(loaderRuns));
  loaderRuns++;
  return source.replace('__LOADER_RUNS__', loaderRuns);
};
