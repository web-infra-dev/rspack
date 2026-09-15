const { one } = require('./shared-1');

it('loads batched splitChunks callbacks for entry d', () => {
  expect(one).toBe(1);
});
