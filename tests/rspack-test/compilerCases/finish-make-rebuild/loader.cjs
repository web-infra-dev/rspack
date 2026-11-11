const path = require('node:path');
module.exports = function (source) {
  this.cacheable(false);
  if (path.basename(this.resourcePath) === 'a.js') {
    if (this.rebuildState.drop) return 'export default "removed";';
    if (this.rebuildState.extra)
      return 'import value from "./extra.js"; export default value;';
  }
  return source;
};
