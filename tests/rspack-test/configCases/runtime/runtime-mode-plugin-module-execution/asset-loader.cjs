module.exports = function (source) {
  this.emitFile('native-assets/addon.txt', source);
  return `
    const path = require('node:path');
    module.exports = require('node:fs').readFileSync(
      path.join(__dirname, __webpack_require__.ab, 'addon.txt'),
      'utf-8',
    ).trim();
  `;
};
