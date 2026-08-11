const { one } = require('./shared-1');
const { two } = require('./shared-2');

it('loads batched splitChunks callbacks for entry a', () => {
  expect(one + two).toBe(3);
});
