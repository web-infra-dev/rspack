module.exports = function (source) {
  if (this.recoveryState.fail) {
    throw new Error('expected module build error');
  }
  return source;
};
