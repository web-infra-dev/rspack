import value from './value';

it('runs an entry named __proto__', () => {
  expect(value).toBe('entry-loaded');
});
