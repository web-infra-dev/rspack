it('reuses nested loader options across tasks without sharing compiler state', () => {
  const values = require('./values');
  expect(values).toHaveLength(128);
  expect(values.every(value => value.mode === EXPECTED_MODE)).toBe(true);
  // Also fail if each task gets a fresh compiler options object.
  expect(values.some(value => value.reused)).toBe(true);
});
