import value from './shared.js';

it('should preserve the shared module', () => {
  expect(value()).toBe(42);
});
