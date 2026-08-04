const value = require('./value');

it('should ignore loader cache markers by default', () => {
  expect(value).toBe(+WATCH_STEP + 1);
});
