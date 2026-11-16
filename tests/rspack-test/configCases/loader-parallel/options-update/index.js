it('updates options while retaining structured-clone-only values', () => {
  expect(require('./values')).toEqual(Array.from({ length: 128 }, (_, i) => i));
});
