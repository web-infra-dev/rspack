const assert = require('node:assert/strict');
const { isMainThread } = require('node:worker_threads');

module.exports = function (_content, _map, data) {
  assert(isMainThread);
  assert.equal(data.self, data);
  assert.equal(data.shared, data.alias);
  assert(data.map instanceof Map);
  assert.equal(data.map.get('shared'), data.shared);
  assert(data.set.has(data.shared));
  assert(Buffer.isBuffer(data.buffer));
  return `module.exports = ${JSON.stringify({ count: data.count, value: data.shared.value, buffer: data.buffer.toString() })}`;
};
