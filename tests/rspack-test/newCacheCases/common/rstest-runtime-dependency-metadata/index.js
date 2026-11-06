import { load as stableLoad } from './stable';
import { load as changedLoad, value } from './changed';

it('restores native request metadata and removes calls after source changes', async () => {
  expect(typeof stableLoad).toBe('function');
  expect(typeof changedLoad).toBe('function');
  expect(value).toBe(COMPILER_INDEX < 2 ? 0 : 1);
  // The new-cache runner performs the initial unchanged restart itself.
  if (COMPILER_INDEX === 1) {
    await NEXT_START();
  }
});
