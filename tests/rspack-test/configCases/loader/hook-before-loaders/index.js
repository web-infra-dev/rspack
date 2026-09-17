it('should run the loader hook once before any loader', () => {
  expect(require('./value')).toBe(42);
});
