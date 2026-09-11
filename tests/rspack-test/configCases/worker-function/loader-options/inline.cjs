const assert = require('node:assert/strict');
module.exports = function () {
  const { fn, local } = this.getOptions();
  assert.equal(this.prepared(0), 42);
  // getOptions and both prepared functions remain synchronous, including ESM exports.
  return `module.exports = ${fn(0) + local(0)}`;
};
