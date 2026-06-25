// This is a noop loader that is used to test the circular dependency plugin.
module.exports = function loader(source) {
  return source;
}
