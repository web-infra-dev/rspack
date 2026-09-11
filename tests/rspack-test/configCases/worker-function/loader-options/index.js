it('restores nested worker functions in main and parallel loader options', () => {
  expect(require('./input')).toBe(42);
});

it('prepares worker functions after a lazy inline ident lookup', () => {
  expect(require('!!./inline.cjs??worker-options!./input')).toBe(42);
});
