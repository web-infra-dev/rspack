const value = require('./value');

it('should combine parallel execution with the loader cache', () => {
  expect(value).toBe(+WATCH_STEP < 2 ? 1 : 2);
});
