module.exports = {
  findBundle: function (i, options) {
    return './main.js';
    return i === 0 ? './main.js' : './module/main.mjs';
  },
};
