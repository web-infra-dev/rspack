import value from './input';

it('should invalidate module cache when a loader build dependency changes', async () => {
  expect(value).toBe(COMPILER_INDEX === 0 ? 'first' : 'second');
  if (COMPILER_INDEX === 0) await NEXT_START();
});
