module.exports = function parallelLoader(source) {
  const callback = this.async();
  setTimeout(() => callback(null, source), 200);
};
