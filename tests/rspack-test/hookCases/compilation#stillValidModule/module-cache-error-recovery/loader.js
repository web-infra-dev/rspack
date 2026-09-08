module.exports = function (source) {
  const options = this.getOptions();
  options.builds++;
  this.addBuildDependency(options.dependency);
  this.cacheable(options.cacheable);
  if (options.fail) throw new Error('expected loader failure');
  return source;
};
