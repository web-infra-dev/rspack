module.exports = function (content) {
  this.getOptions().builtModules.push(this.resourcePath);
  if (content.includes('export default 1')) {
    this.emitWarning(new Error('stale module warning'));
  }
  return content;
};
