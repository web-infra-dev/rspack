const used = require('./barrel').used;

it('keeps the selected CommonJS star reexport', () => {
  expect(used).toBe('used');
});
