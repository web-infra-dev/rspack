const path = require('node:path');

// Mimics tailwind-style css generation: the stylesheet depends on other
// source files, so editing them rebuilds the css module even when the
// emitted css stays byte-identical.
module.exports = function (source) {
  this.addDependency(path.resolve(__dirname, 'src/dep.txt'));
  return source;
};
