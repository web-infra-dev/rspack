module.exports = function loader(source) {
  const { emitWarning } = this;
  emitWarning("This is a warning");
  return source;
};
