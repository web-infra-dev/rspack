it('preserves same-stage main/worker order, before, mutation and bail', () => {
  expect(require('./original')).toBe(42);
  expect(() => require('./ignored')).toThrow(/Cannot find module/);
});
