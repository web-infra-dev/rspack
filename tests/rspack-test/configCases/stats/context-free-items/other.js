import value from './shared.js';

it('should preserve the second entry', () => {
  expect(value() + 1).toBe(43);
});
