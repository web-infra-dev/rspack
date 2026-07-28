let loaderRuns = 0;

module.exports = function () {
  this.addDependency(`${this.rootContext}/trigger.js`);
  loaderRuns++;
  return `module.exports = ${loaderRuns};`;
};
