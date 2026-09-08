import value from './stable';
import round from './trigger';

it('persists mutations made to a module restored from disk', async () => {
  expect(value).toBe(42);
  expect(round).toBe(COMPILER_INDEX);
  if (COMPILER_INDEX < 2) await NEXT_START();
});
