import value from 'linked/value.js';

it('loads a symlink target outside the package that describes the link', () => {
  expect(value).toBe(42);
});
