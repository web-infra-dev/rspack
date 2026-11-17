module.exports = function (source) {
  if (this.resourcePath.endsWith("parallel-crash-resource.js")) process.exit(42);
  const callback = this.async();
  setTimeout(() => callback(null, source), 10);
};
