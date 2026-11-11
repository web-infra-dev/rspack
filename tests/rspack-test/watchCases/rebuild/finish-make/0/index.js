const value = require('./dep');

it('should rebuild the entry in finishMake after changing a dependency', () => {
  expect(value).toBe(Number(WATCH_STEP));
});
