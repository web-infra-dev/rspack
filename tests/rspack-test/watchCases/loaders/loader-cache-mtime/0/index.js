const value = require('./value');

it('should not cache when the resource mtime is not safely before the attempt', () => {
  expect(value).toBe(+WATCH_STEP + 1);
});
