import value from './value';
it('should preserve module behavior with shared accessors', () => {
  expect(value).toBe(42);
});
