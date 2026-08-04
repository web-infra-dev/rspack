const value = require('./value');

it('should not cache a non-cacheable loader segment', () => {
  expect(value).toBe(+WATCH_STEP + 1);
});
