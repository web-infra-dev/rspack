module.exports = function () {
  this.cacheable(false);
  return `.text { color: rgb(${this.rebuildState.phase}, 0, 0); }`;
};
