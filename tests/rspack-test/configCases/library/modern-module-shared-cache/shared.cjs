exports.child = require('./child.cjs');
exports.cached = () => require.cache[require.resolve('./child.cjs')].exports;
exports.cache = require.cache;
exports.reload = () => {
  delete require.cache[require.resolve('./child.cjs')];
  return require('./child.cjs');
};
exports.setCache = cache => { __webpack_require__.c = cache; };
