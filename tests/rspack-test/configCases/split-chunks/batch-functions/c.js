const { one } = require('./shared-1');
const { two } = require('./shared-2');

it('loads batched splitChunks callbacks for entry c', () => {
  expect(one + two).toBe(3);
});
