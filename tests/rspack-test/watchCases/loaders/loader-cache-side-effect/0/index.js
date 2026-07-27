const value = require('./value');

it('should not cache a loader segment with non-replayable side effects', () => {
  expect(value).toBe(+WATCH_STEP + 1);
});
