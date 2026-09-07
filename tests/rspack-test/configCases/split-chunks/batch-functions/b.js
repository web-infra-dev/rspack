const { one } = require('./shared-1');
const { two } = require('./shared-2');
const { three } = require('./shared-3');

it('loads batched splitChunks callbacks for entry b', () => {
  expect(one + two + three).toBe(6);
});
