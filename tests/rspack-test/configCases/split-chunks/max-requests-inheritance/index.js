import initial from './shared.js?initial';

it('should preserve initial and async modules with request limits', async () => {
  expect(initial.value).toBe(42);
  const { default: asyncValue } = await import('./async.js');
  expect(asyncValue.value).toBe(42);
});
