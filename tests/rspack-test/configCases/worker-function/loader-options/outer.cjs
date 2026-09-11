const assert = require('node:assert/strict');
module.exports = (_value, options) => {
  assert.equal(options.self, options);
  assert.equal(options.same, options.nested);
  assert.equal(options.map.get('fn'), options.nested);
  assert.equal([...options.set][0], options.nested);
  return options.nested(0);
};
