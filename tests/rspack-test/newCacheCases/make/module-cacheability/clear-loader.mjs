export default function (source) {
  this.cacheable(false);
  this.clearDependencies();
  this.addDependency(this.resourcePath);
  return source;
};
