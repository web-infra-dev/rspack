module.exports = function () {
  const { require: loader, requireScope } = this._compiler.rspack.RuntimeGlobals;
  return `module.exports = {
    read() { return __webpack_require__.pluginValue; },
    increment(key) { __webpack_require__[key]++; },
    writeScope(value) { ${requireScope}.pluginValue = value; },
    readPath() { return ${loader}.p; },
    writePath(value) { ${loader}.p = value; }
  };`;
};
