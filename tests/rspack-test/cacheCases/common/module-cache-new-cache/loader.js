module.exports = function (content) {
  this.getOptions().builtModules.push(this.resourcePath);
  return content;
};
