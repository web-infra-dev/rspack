import value from './value';

it('should reuse the cached result of following loaders', () => {
  expect(value).toBe(1);
});
