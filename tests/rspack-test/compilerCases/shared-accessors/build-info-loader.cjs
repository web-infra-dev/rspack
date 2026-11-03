module.exports = function (source) {
  this.addDependency(this.getOptions().dependency);
  return source;
};
