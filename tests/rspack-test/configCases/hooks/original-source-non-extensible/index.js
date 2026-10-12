import value from './value';

it('preserves output with non-extensible module wrappers', () => {
  expect(value).toBe(42);
});
