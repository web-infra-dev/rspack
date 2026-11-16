const assert = require('node:assert/strict');
module.exports = function () {
  const options = this.getOptions();
  const expected = Number(this.resourceQuery.slice(1));
  assert.equal(options.revision, expected);
  if (expected % 2 === 0) assert.equal(options.shared.byteLength, 4);
  else assert.equal(options.shared, undefined);
  return `module.exports = ${options.revision};`;
};
