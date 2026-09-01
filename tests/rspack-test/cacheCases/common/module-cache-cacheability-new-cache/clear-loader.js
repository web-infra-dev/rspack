module.exports = function (source) {
  this.cacheable(false);
  this.clearDependencies();
  this.addDependency(this.resourcePath);
  return source;
};
