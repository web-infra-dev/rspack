it('should expose type exports to the following JS loader', () => {
  const { value } = require('./lib');
  expect(value).toBe(1);
});
