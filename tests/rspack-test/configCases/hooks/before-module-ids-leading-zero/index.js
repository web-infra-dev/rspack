const context = require.context('./', false, /lazy-a\.js$/, 'lazy');

it('should preserve a leading-zero string module id', async () => {
  const module = await context('./lazy-a.js');
  expect(module.default).toBe('lazy-a');
});
