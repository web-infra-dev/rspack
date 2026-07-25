const { local, shadowed, used } = require('./barrel');

it('keeps explicit exports and the selected CommonJS star reexport', () => {
  expect(local).toBe('local');
  expect(shadowed).toBe('barrel');
  expect(used).toBe('used');
});
