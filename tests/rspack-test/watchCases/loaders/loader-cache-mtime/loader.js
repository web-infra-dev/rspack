let runs = 0;

module.exports = function (source) {
  this.addDependency(`${this.rootContext}/trigger.js`);
  runs++;
  return source.replace('__RUNS__', runs);
};
