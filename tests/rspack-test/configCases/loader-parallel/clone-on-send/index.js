it('should serialize loader input only when sending it to the worker', () => {
  expect(require('./lib.js')).toBe(1);
});
