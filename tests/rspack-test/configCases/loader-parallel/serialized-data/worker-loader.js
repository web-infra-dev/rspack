const assert = require('node:assert/strict');
const { isMainThread } = require('node:worker_threads');

module.exports = function (content, map, data) {
  assert.equal(isMainThread, false);
  const options = this.getOptions();
  assert.equal(options.transport, 'native bytes');
  for (const value of [options, data]) {
    assert.equal(value.self, value);
    assert.equal(value.shared, value.alias);
    assert(value.map instanceof Map);
    assert.equal(value.map.get('shared'), value.shared);
    assert(value.set instanceof Set);
    assert(value.set.has(value.shared));
    assert(Buffer.isBuffer(value.buffer));
    assert(value.typed instanceof Uint16Array);
    assert.deepEqual([...value.typed], [256, 1024]);
    assert.equal(value.bigint, 12345678901234567890n);
    assert(Object.hasOwn(value, 'missing'));
    assert.equal(value.missing, undefined);
  }
  assert(options.date instanceof Date);
  assert.equal(options.date.toISOString(), '2020-01-02T00:00:00.000Z');
  assert(options.regexp instanceof RegExp);
  assert.equal(options.regexp.source, 'loader');
  assert.equal(options.regexp.flags, 'gi');
  assert.equal(options.buffer.toString(), 'options');
  assert.equal(data.buffer.toString(), 'additional data');
  data.count++;
  data.shared.value++;
  this.callback(null, content, map, data);
};
