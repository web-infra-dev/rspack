module.exports = function loader(source) {
  const { emitWarning } = this;
  emitWarning("Generated Warning");
  return source;
};
