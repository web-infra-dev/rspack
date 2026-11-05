module.exports = function (source) {
  return source.padEnd(this.getOptions().size, ' ');
};
