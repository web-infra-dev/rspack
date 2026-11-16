module.exports = function () {
  return `module.exports = {
    read() { return __webpack_require__.pluginValue; },
    increment(key) { __webpack_require__[key]++; }
  };`;
};
