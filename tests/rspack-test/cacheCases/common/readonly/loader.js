module.exports = function (content) {
  const options = this.getOptions();
  options.count++;
  return content;
};
