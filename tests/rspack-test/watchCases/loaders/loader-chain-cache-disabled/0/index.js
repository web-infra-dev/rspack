const value = require('./value');

it('should not store a loader chain when cacheable(false) is called', () => {
  expect(value).toBe(+WATCH_STEP + 1);
});
